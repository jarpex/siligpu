use clap::{ArgGroup, Parser};
use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    dictionary::{CFDictionary, CFDictionaryGetValue, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
use std::{
    error::Error,
    os::raw::c_void,
    ptr::null,
    thread::sleep,
    time::Duration,
};

#[derive(Parser)]
#[command(
    name = "siligpu",
    about = "Apple Silicon GPU Usage Display Utility for macOS",
    version = env!("CARGO_PKG_VERSION"),
)]
#[command(group(
    ArgGroup::new("mode")
        .args(&["verbose", "summary", "value_only"])
        .multiple(false),
))]
struct Args {
    /// Verbose mode (default) – show detailed performance states
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Summary mode – show one-line summary (e.g., "Usage: 10.25%")
    #[arg(short = 's', long = "summary")]
    summary: bool,

    /// Quiet mode – output only the numeric value (e.g., "12.34%")
    #[arg(short = 'q', long = "value-only")]
    value_only: bool,

    /// Time between samples
    /// Accepts plain numbers (ms) or units: ms, s, m, h. (e.g. `100`, `100ms`, `1s`, `1m`, `1h`).
    #[arg(short = 't', long = "time", default_value = "1000ms")]
    time: String,
}

enum Mode {
    Verbose,
    Summary,
    ValueOnly,
}

/// Parse strings like "100", "100ms", "1s", "1m", "1h" into a `Duration`.
fn parse_time_arg(s: &str) -> Result<Duration, Box<dyn Error>> {
    let s = s.trim();
    if let Some(num) = s.strip_suffix("ms") {
        let ms: u64 = num.parse()?;
        Ok(Duration::from_millis(ms))
    } else if let Some(num) = s.strip_suffix('s') {
        let secs: u64 = num.parse()?;
        Ok(Duration::from_secs(secs))
    } else if let Some(num) = s.strip_suffix('m') {
        let mins: u64 = num.parse()?;
        Ok(Duration::from_secs(mins * 60))
    } else if let Some(num) = s.strip_suffix('h') {
        let hours: u64 = num.parse()?;
        Ok(Duration::from_secs(hours * 3600))
    } else {
        // No unit → milliseconds
        let ms: u64 = s.parse()?;
        Ok(Duration::from_millis(ms))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mode = if args.summary {
        Mode::Summary
    } else if args.value_only {
        Mode::ValueOnly
    } else {
        Mode::Verbose
    };

    // Parse the user-provided interval (or use 1000 ms by default)
    let sample_interval = parse_time_arg(&args.time)
        .map_err(|e| format!("Invalid time '{}': {}", args.time, e))?;

    // 1. Create CFString instances for group and subgroup identifiers
    let group_cf = CFString::new("GPU Stats");
    let subgroup_cf = CFString::new("GPU Performance States");

    // 2. Copy all channels in the specified group/subgroup
    let chans_raw = unsafe {
        IOReportCopyChannelsInGroup(
            group_cf.as_concrete_TypeRef(),
            subgroup_cf.as_concrete_TypeRef(),
            0,
            0,
            0,
        )
    };
    if chans_raw.is_null() {
        return Err("Failed to get channels".into());
    }
    let channels: CFDictionary<CFString, CFType> =
        unsafe { CFDictionary::wrap_under_create_rule(chans_raw) };

    // 3. Create a subscription to the selected channels
    let mut sub_ref: CFDictionaryRef = null();
    let subscription = unsafe {
        IOReportCreateSubscription(
            null(),
            channels.as_concrete_TypeRef(),
            &mut sub_ref,
            0,
            null(),
        )
    };
    if subscription.is_null() {
        return Err("Failed to create subscription".into());
    }

    // 4. Take two samples separated by the user-defined interval
    let sample1 = unsafe { IOReportCreateSamples(subscription, channels.as_concrete_TypeRef(), null()) };
    sleep(sample_interval);
    let sample2 = unsafe { IOReportCreateSamples(subscription, channels.as_concrete_TypeRef(), null()) };
    let delta_raw = unsafe { IOReportCreateSamplesDelta(sample1, sample2, null()) };
    let delta: CFDictionary<CFString, CFType> =
        unsafe { CFDictionary::wrap_under_create_rule(delta_raw) };

    // 5. Extract the array of channel dictionaries from the delta
    let key_cf = CFString::new("IOReportChannels");
    let arr_ref: CFArrayRef = unsafe {
        CFDictionaryGetValue(
            delta.as_concrete_TypeRef(),
            key_cf.as_concrete_TypeRef() as CFTypeRef,
        ) as CFArrayRef
    };
    let channel_array: CFArray<CFDictionary<CFString, CFType>> =
        unsafe { CFArray::wrap_under_get_rule(arr_ref) };

    // 6. Iterate over each channel dictionary in the array
    for dict_wrapper in channel_array.iter() {
        let dict_ref = dict_wrapper.as_CFTypeRef() as CFDictionaryRef;
        let dict: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_get_rule(dict_ref) };

        let grp_name = unsafe {
            CFString::wrap_under_get_rule(IOReportChannelGetGroup(dict.as_concrete_TypeRef()))
        }
        .to_string();
        let subgrp_name = unsafe {
            CFString::wrap_under_get_rule(IOReportChannelGetSubGroup(dict.as_concrete_TypeRef()))
        }
        .to_string();

        if grp_name != "GPU Stats" || subgrp_name != "GPU Performance States" {
            continue;
        }

        // Count and sum residency times for each performance state
        let state_count = unsafe { IOReportStateGetCount(dict.as_concrete_TypeRef()) };
        let mut total: i64 = 0;
        let mut active: i64 = 0;

        for idx in 0..state_count {
            let state_name = unsafe {
                CFString::wrap_under_get_rule(
                    IOReportStateGetNameForIndex(dict.as_concrete_TypeRef(), idx),
                )
            }
            .to_string();
            let residency = unsafe { IOReportStateGetResidency(dict.as_concrete_TypeRef(), idx) };

            total += residency;
            if !state_name.contains("IDLE")
                && !state_name.contains("OFF")
                && !state_name.contains("DOWN")
            {
                active += residency;
            }
        }

        let usage = (active as f64 / total.max(1) as f64) * 100.0;

        match mode {
            Mode::Verbose => {
                println!("{:>0} / {:<0}", grp_name, subgrp_name);
                for idx in 0..state_count {
                    let state_name = unsafe {
                        CFString::wrap_under_get_rule(
                            IOReportStateGetNameForIndex(dict.as_concrete_TypeRef(), idx),
                        )
                    }
                    .to_string();
                    let residency = unsafe { IOReportStateGetResidency(dict.as_concrete_TypeRef(), idx) };
                    println!("  {:>6}: {:>21} µs", state_name, residency);
                }
                println!("  {:>15}: {:>12} µs (active)", "→ Total active", active);
                println!("  {:>15}: {:>12} µs (total)", "→ Total", total);
                println!("  {:>15}: {:>12.2} %", "→ Usage", usage);
            }
            Mode::Summary => {
                println!("Usage: {:>6.2}%", usage);
            }
            Mode::ValueOnly => {
                println!("{:.2}%", usage);
            }
        }
    }

    Ok(())
}

// Bindings for IOReport functions
#[link(name = "IOReport", kind = "dylib")]
extern "C" {
    fn IOReportCopyChannelsInGroup(
        group: CFStringRef,
        subgroup: CFStringRef,
        flags: u64,
        a: u64,
        b: u64,
    ) -> CFDictionaryRef;

    fn IOReportCreateSubscription(
        allocator: *const c_void,
        channels: CFDictionaryRef,
        out: *mut CFDictionaryRef,
        flags: u64,
        unknown: CFTypeRef,
    ) -> *const c_void;

    fn IOReportCreateSamples(
        sub: *const c_void,
        channels: CFDictionaryRef,
        unknown: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportCreateSamplesDelta(
        s1: CFDictionaryRef,
        s2: CFDictionaryRef,
        unknown: CFTypeRef,
    ) -> CFDictionaryRef;

    fn IOReportChannelGetGroup(item: CFDictionaryRef) -> CFStringRef;
    fn IOReportChannelGetSubGroup(item: CFDictionaryRef) -> CFStringRef;

    fn IOReportStateGetCount(item: CFDictionaryRef) -> i32;
    fn IOReportStateGetNameForIndex(item: CFDictionaryRef, index: i32) -> CFStringRef;
    fn IOReportStateGetResidency(item: CFDictionaryRef, index: i32) -> i64;
}
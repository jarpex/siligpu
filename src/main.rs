use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef},
    base::{CFRelease, CFTypeRef},
    dictionary::{CFDictionaryGetValue, CFDictionaryRef},
    string::{CFStringCreateWithCString, CFStringGetCString, CFStringRef, kCFStringEncodingUTF8},
};

use std::{ffi::CStr, os::raw::c_void, ptr::null};

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
    fn IOReportChannelGetChannelName(item: CFDictionaryRef) -> CFStringRef;

    fn IOReportStateGetCount(item: CFDictionaryRef) -> i32;
    fn IOReportStateGetNameForIndex(item: CFDictionaryRef, index: i32) -> CFStringRef;
    fn IOReportStateGetResidency(item: CFDictionaryRef, index: i32) -> i64;
}

fn cfstring(s: &str) -> CFStringRef {
    let cstr = std::ffi::CString::new(s).unwrap();
    unsafe {
        CFStringCreateWithCString(
            null(),
            cstr.as_ptr(),
            kCFStringEncodingUTF8,
        )
    }
}

fn from_cfstring(s: CFStringRef) -> String {
    unsafe {
        let mut buf = [0u8; 256];
        let ok = CFStringGetCString(
            s,
            buf.as_mut_ptr() as *mut i8,
            buf.len() as isize,
            kCFStringEncodingUTF8,
        );
        if ok == 0 {
            "<invalid>".into()
        } else {
            let cstr = CStr::from_ptr(buf.as_ptr() as *const i8);
            cstr.to_string_lossy().into()
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let group = cfstring("GPU Stats");
        let subgroup = cfstring("GPU Performance States");

        let chans = IOReportCopyChannelsInGroup(group, subgroup, 0, 0, 0);
        if chans.is_null() {
            return Err("Failed to get channels".into());
        }

        let mut out: CFDictionaryRef = std::mem::zeroed();
        let sub = IOReportCreateSubscription(null(), chans, &mut out, 0, null());

        let s1 = IOReportCreateSamples(sub, chans, null());
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let s2 = IOReportCreateSamples(sub, chans, null());
        let delta = IOReportCreateSamplesDelta(s1, s2, null());

        let key = cfstring("IOReportChannels");
        let arr = CFDictionaryGetValue(delta, key as _) as CFArrayRef;
        let count = CFArrayGetCount(arr);

        println!("GPU Residency (per frequency state):\n");

        for i in 0..count {
            let item = CFArrayGetValueAtIndex(arr, i) as CFDictionaryRef;

            let group = from_cfstring(IOReportChannelGetGroup(item));
            let subgroup = from_cfstring(IOReportChannelGetSubGroup(item));
            let name = from_cfstring(IOReportChannelGetChannelName(item));

            if group != "GPU Stats" || subgroup != "GPU Performance States" {
                continue;
            }

            println!("{:<10} / {:<25} / {}", group, subgroup, name);

            let state_count = IOReportStateGetCount(item);
            let mut total = 0i64;
            let mut active = 0i64;

            for j in 0..state_count {
                let sname = from_cfstring(IOReportStateGetNameForIndex(item, j));
                let val = IOReportStateGetResidency(item, j);
                println!("  {:>15}: {:>10} µs", sname, val);
                total += val;
                if !sname.contains("IDLE") && !sname.contains("OFF") && !sname.contains("DOWN") {
                    active += val;
                }
            }

            println!("  {:>15}: {:>10} µs (active)", "→ Total active", active);
            println!("  {:>15}: {:>10} µs (total)", "→ Total", total);
            println!(
                "  {:>15}: {:>6.2} %\n",
                "→ Usage",
                (active as f64 / total.max(1) as f64) * 100.0
            );
        }

        // cleanup
        CFRelease(group as _);
        CFRelease(subgroup as _);
        CFRelease(chans as _);
        CFRelease(s1 as _);
        CFRelease(s2 as _);
        CFRelease(delta as _);
    }

    Ok(())
}
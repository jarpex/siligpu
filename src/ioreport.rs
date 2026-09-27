use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    dictionary::{CFDictionary, CFDictionaryGetValue, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
use core_foundation_sys::base::CFRelease;
use serde::Serialize;
use std::{fmt, os::raw::c_void, ptr};

/// A single GPU performance state.
#[derive(Debug, Serialize)]
pub struct GPUState {
    pub name: String,
    #[serde(rename = "residency_micros")]
    pub residency: i64,
    pub is_active: bool,
}

/// A channel of GPU statistics containing multiple states.
#[derive(Debug, Serialize)]
pub struct GPUChannel {
    pub group: String,
    pub subgroup: String,
    pub states: Vec<GPUState>,
}

impl GPUChannel {
    #[must_use]
    pub fn total_residency(&self) -> i64 {
        self.states.iter().map(|s| s.residency).sum()
    }

    #[must_use]
    pub fn active_residency(&self) -> i64 {
        self.states
            .iter()
            .filter(|s| s.is_active)
            .map(|s| s.residency)
            .sum()
    }

    /// Calculates the percentage of time the GPU was active.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn usage(&self) -> f64 {
        let total = self.total_residency();
        if total == 0 {
            0.0
        } else {
            (self.active_residency() as f64 / total as f64) * 100.0
        }
    }
}

/// A wrapper around the IOReport library.
pub struct IOReport {
    subscription: IOReportSubscriptionRef,
    channels: CFDictionary<CFString, CFType>,
}

impl fmt::Debug for IOReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IOReport")
            .field("subscription", &self.subscription)
            .field("channels", &"<CFDictionary>")
            .finish()
    }
}

/// Errors that can occur while interacting with IOReport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IOReportError {
    ChannelsUnavailable,
    SubscriptionFailed,
    SampleFailed,
    DeltaFailed,
    MissingChannelArray,
}

impl fmt::Display for IOReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChannelsUnavailable => write!(f, "IOReport channels unavailable"),
            Self::SubscriptionFailed => write!(f, "Failed to create IOReport subscription"),
            Self::SampleFailed => write!(f, "Failed to capture IOReport sample"),
            Self::DeltaFailed => write!(f, "Failed to compute IOReport sample delta"),
            Self::MissingChannelArray => write!(f, "IOReport response missing channel data"),
        }
    }
}

impl std::error::Error for IOReportError {}

type IOReportSubscriptionRef = *const c_void;

unsafe fn cf_string_to_string(ptr: CFStringRef) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CFString::wrap_under_get_rule(ptr).to_string()
    }
}

fn contains_ignore_case(haystack: &str, needle: &str) -> bool {
    let needle = needle.as_bytes();
    haystack
        .as_bytes()
        .windows(needle.len())
        .any(|w| w.eq_ignore_ascii_case(needle))
}

impl IOReport {
    /// Creates a new subscription for the requested IOReport group and subgroup.
    pub fn new(group_name: &str, subgroup_name: &str) -> Result<Self, IOReportError> {
        let group_cf = CFString::new(group_name);
        let subgroup_cf = CFString::new(subgroup_name);

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
            return Err(IOReportError::ChannelsUnavailable);
        }

        let channels: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_create_rule(chans_raw) };

        let mut sub_ref: CFDictionaryRef = ptr::null();
        let subscription = unsafe {
            IOReportCreateSubscription(
                ptr::null(),
                channels.as_concrete_TypeRef(),
                &raw mut sub_ref,
                0,
                ptr::null(),
            )
        };

        // Release the merged channels dictionary if created
        if !sub_ref.is_null() {
            unsafe {
                CFRelease(sub_ref as CFTypeRef);
            }
        }

        if subscription.is_null() {
            return Err(IOReportError::SubscriptionFailed);
        }

        Ok(Self {
            subscription,
            channels,
        })
    }

    /// Captures a single IOReport sample.
    pub fn sample(&self) -> Result<CFDictionary<CFString, CFType>, IOReportError> {
        unsafe {
            let raw = IOReportCreateSamples(
                self.subscription,
                self.channels.as_concrete_TypeRef(),
                ptr::null(),
            );

            if raw.is_null() {
                return Err(IOReportError::SampleFailed);
            }

            Ok(CFDictionary::wrap_under_create_rule(raw))
        }
    }

    /// Computes the delta between two samples and extracts GPU channel data.
    pub fn get_delta(
        sample1: &CFDictionary<CFString, CFType>,
        sample2: &CFDictionary<CFString, CFType>,
    ) -> Result<Vec<GPUChannel>, IOReportError> {
        let delta_raw = unsafe {
            IOReportCreateSamplesDelta(
                sample1.as_concrete_TypeRef(),
                sample2.as_concrete_TypeRef(),
                ptr::null(),
            )
        };

        if delta_raw.is_null() {
            return Err(IOReportError::DeltaFailed);
        }

        let delta: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_create_rule(delta_raw) };

        let key_cf = CFString::new("IOReportChannels");

        #[allow(clippy::cast_ptr_alignment)]
        let arr_ref: CFArrayRef = unsafe {
            CFDictionaryGetValue(
                delta.as_concrete_TypeRef(),
                key_cf.as_concrete_TypeRef() as CFTypeRef,
            ) as CFArrayRef
        };

        if arr_ref.is_null() {
            return Err(IOReportError::MissingChannelArray);
        }

        let channel_array: CFArray<CFDictionary<CFString, CFType>> =
            unsafe { CFArray::wrap_under_get_rule(arr_ref) };

        let mut results = Vec::new();

        for dict in channel_array.iter() {
            let dict_ref = dict.as_concrete_TypeRef();

            let grp_name = unsafe { cf_string_to_string(IOReportChannelGetGroup(dict_ref)) };
            let subgrp_name = unsafe { cf_string_to_string(IOReportChannelGetSubGroup(dict_ref)) };
            let unit = unsafe { cf_string_to_string(IOReportChannelGetUnitLabel(dict_ref)) };

            let state_count = unsafe { IOReportStateGetCount(dict_ref) };
            let mut states = Vec::new();

            for idx in 0..state_count {
                let state_name =
                    unsafe { cf_string_to_string(IOReportStateGetNameForIndex(dict_ref, idx)) };

                let raw_residency = unsafe { IOReportStateGetResidency(dict_ref, idx) };

                // Convert 24Mticks to microseconds
                let residency = if unit.trim() == "24Mticks" {
                    raw_residency / 24
                } else {
                    raw_residency
                };

                let is_active = !contains_ignore_case(&state_name, "idle")
                    && !contains_ignore_case(&state_name, "off")
                    && !contains_ignore_case(&state_name, "down");

                states.push(GPUState {
                    name: state_name,
                    residency,
                    is_active,
                });
            }

            results.push(GPUChannel {
                group: grp_name,
                subgroup: subgrp_name,
                states,
            });
        }

        Ok(results)
    }
}

impl Drop for IOReport {
    fn drop(&mut self) {
        if !self.subscription.is_null() {
            unsafe {
                CFRelease(self.subscription as CFTypeRef);
            }
        }
    }
}

#[allow(non_snake_case)]
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
    fn IOReportChannelGetUnitLabel(item: CFDictionaryRef) -> CFStringRef;

    fn IOReportStateGetCount(item: CFDictionaryRef) -> i32;
    fn IOReportStateGetNameForIndex(item: CFDictionaryRef, index: i32) -> CFStringRef;
    fn IOReportStateGetResidency(item: CFDictionaryRef, index: i32) -> i64;
}

#[cfg(test)]
#[allow(clippy::result_large_err, clippy::float_cmp)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn arb_gpu_state() -> impl Strategy<Value = GPUState> {
        ("[a-zA-Z0-9]{1,10}", 0..10_000_000i64, any::<bool>()).prop_map(
            |(name, residency, is_active)| GPUState {
                name,
                residency,
                is_active,
            },
        )
    }

    proptest! {
        #[test]
        fn test_channel_math_invariants(
            states in prop::collection::vec(arb_gpu_state(), 0..10)
        ) {
            let channel = GPUChannel {
                group: "FuzzGroup".into(),
                subgroup: "FuzzSubgroup".into(),
                states,
            };

            let total = channel.total_residency();
            let active = channel.active_residency();

            prop_assert!(active <= total,
                "Active {} > Total {}", active, total);

            let usage = channel.usage();
            prop_assert!(usage >= 0.0, "Usage < 0: {}", usage);
            prop_assert!(usage <= 100.0, "Usage > 100: {}", usage);

            if total == 0 {
                prop_assert_eq!(usage, 0.0);
            }
        }
    }
}

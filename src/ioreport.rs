use core_foundation::{
    array::{CFArray, CFArrayRef},
    base::{CFType, CFTypeRef, TCFType},
    dictionary::{CFDictionary, CFDictionaryGetValue, CFDictionaryRef},
    string::{CFString, CFStringRef},
};
use core_foundation_sys::base::CFRelease;
use serde::Serialize;
use std::{fmt, os::raw::c_void, ptr::null};

/// Represents a single GPU performance state (e.g., "P1", "IDLE").
#[derive(Debug, Serialize)]
pub struct GPUState {
    /// The name of the state (e.g., "P1", "IDLE").
    pub name: String,
    /// The time spent in this state in microseconds.
    pub residency: i64,
    /// Whether this state is considered "active" (i.e., not IDLE, OFF, or DOWN).
    pub is_active: bool,
}

/// Represents a channel of GPU statistics, containing multiple states.
#[derive(Debug, Serialize)]
pub struct GPUChannel {
    /// The group name (e.g., "GPU Stats").
    pub group: String,
    /// The subgroup name (e.g., "GPU Performance States").
    pub subgroup: String,
    /// The list of performance states in this channel.
    pub states: Vec<GPUState>,
}

impl GPUChannel {
    /// Calculates the total residency time across all states in this channel.
    pub fn total_residency(&self) -> i64 {
        self.states.iter().map(|s| s.residency).sum()
    }

    /// Calculates the total residency time for active states only.
    pub fn active_residency(&self) -> i64 {
        self.states
            .iter()
            .filter(|s| s.is_active)
            .map(|s| s.residency)
            .sum()
    }

    /// Calculates the percentage of time the GPU was active.
    pub fn usage(&self) -> f64 {
        let total = self.total_residency();
        if total == 0 {
            0.0
        } else {
            (self.active_residency() as f64 / total as f64) * 100.0
        }
    }
}

/// A wrapper around the IOReport library for querying system statistics.
pub struct IOReport {
    subscription: IOReportSubscriptionRef,
    channels: CFDictionary<CFString, CFType>,
}

/// Errors that can occur while interacting with IOReport.
#[derive(Debug)]
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
            IOReportError::ChannelsUnavailable => write!(f, "IOReport channels unavailable"),
            IOReportError::SubscriptionFailed => write!(f, "Failed to create IOReport subscription"),
            IOReportError::SampleFailed => write!(f, "Failed to capture IOReport sample"),
            IOReportError::DeltaFailed => write!(f, "Failed to compute IOReport sample delta"),
            IOReportError::MissingChannelArray => write!(f, "IOReport response missing channel data"),
        }
    }
}

impl std::error::Error for IOReportError {}

// Opaque type for the subscription
type IOReportSubscriptionRef = *const c_void;

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
                null(),
            );

            if raw.is_null() {
                return Err(IOReportError::SampleFailed);
            }

            Ok(CFDictionary::wrap_under_create_rule(raw))
        }
    }

    pub fn get_delta(
        sample1: &CFDictionary<CFString, CFType>,
        sample2: &CFDictionary<CFString, CFType>,
    ) -> Result<Vec<GPUChannel>, IOReportError> {
        let delta_raw = unsafe {
            IOReportCreateSamplesDelta(
                sample1.as_concrete_TypeRef(),
                sample2.as_concrete_TypeRef(),
                null(),
            )
        };

        if delta_raw.is_null() {
            return Err(IOReportError::DeltaFailed);
        }

        let delta: CFDictionary<CFString, CFType> =
            unsafe { CFDictionary::wrap_under_create_rule(delta_raw) };

        let key_cf = CFString::new("IOReportChannels");
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

            let state_count = unsafe { IOReportStateGetCount(dict.as_concrete_TypeRef()) };
            let mut states = Vec::new();

            for idx in 0..state_count {
                let state_name = unsafe {
                    CFString::wrap_under_get_rule(IOReportStateGetNameForIndex(
                        dict.as_concrete_TypeRef(),
                        idx,
                    ))
                }
                .to_string();
                let residency =
                    unsafe { IOReportStateGetResidency(dict.as_concrete_TypeRef(), idx) };
                let upper_state = state_name.to_ascii_uppercase();
                let is_active = !upper_state.contains("IDLE")
                    && !upper_state.contains("OFF")
                    && !upper_state.contains("DOWN");

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
                CFRelease(self.subscription);
            }
        }
    }
}

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
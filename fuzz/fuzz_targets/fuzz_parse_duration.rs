#![no_main]

use libfuzzer_sys::fuzz_target;
use siligpu::parse_duration;

fuzz_target!(|data: &str| {
    let _ = parse_duration(data);
    
    if let Ok(duration) = parse_duration(data) {
        let _ = duration.as_nanos();
    }
});
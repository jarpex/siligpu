// Integration tests for the GPU channel logic.
// Runs as part of `cargo test` and uses the public crate API.

use siligpu::ioreport::{GPUChannel, GPUState};

const EPS: f64 = 1e-6;

#[test]
fn integration_gpu_channel_usage() {
    let states = vec![
        GPUState { name: "IDLE".to_string(), residency: 100, is_active: false },
        GPUState { name: "P1".to_string(), residency: 50, is_active: true },
        GPUState { name: "P2".to_string(), residency: 50, is_active: true },
    ];

    let channel = GPUChannel { group: "Test".to_string(), subgroup: "Test".to_string(), states };

    assert_eq!(channel.total_residency(), 200);
    assert_eq!(channel.active_residency(), 100);
    assert!((channel.usage() - 50.0).abs() < EPS, "usage expected ~50.0, got {}", channel.usage());
}

#[test]
fn integration_gpu_channel_usage_zero_total() {
    let channel = GPUChannel { group: "Test".to_string(), subgroup: "Test".to_string(), states: vec![] };
    assert!((channel.usage() - 0.0).abs() < EPS, "usage expected 0.0, got {}", channel.usage());
}

#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    warnings,
    missing_debug_implementations,
    unreachable_pub,
    rust_2018_idioms,
    unused_lifetimes,
    non_ascii_idents
)]
#![warn(clippy::nursery, clippy::cargo)]
#![allow(
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::struct_excessive_bools,
    clippy::multiple_crate_versions,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::wildcard_imports,
    clippy::option_if_let_else
)]

use anyhow::{Context, Result};
use clap::{ArgGroup, Parser};
use std::thread::sleep;
use std::time::Duration;

use siligpu::ioreport::{GPUChannel, IOReport};
use siligpu::parse_duration;

const GROUP: &str = "GPU Stats";
const SUBGROUP: &str = "GPU Performance States";

#[derive(Parser, Debug)]
#[command(
    name = "siligpu",
    about = "Apple Silicon GPU Usage Display Utility for macOS",
    version = env!("CARGO_PKG_VERSION")
)]
#[command(group(
    ArgGroup::new("mode")
        .args(&["verbose", "summary", "value_only", "json"])
        .multiple(false)
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

    /// JSON mode – output results in JSON format
    #[arg(short = 'j', long = "json")]
    json: bool,

    /// Time between samples (e.g., 100, 100ms, 1s, 1m, 1h)
    #[arg(short = 't', long = "time", default_value = "1000ms", value_parser = parse_duration)]
    time: Duration,
}

fn print_channel(channel: &GPUChannel, args: &Args) -> Result<()> {
    let usage = channel.usage();

    if args.json {
        let json_output = serde_json::json!({
            "usage_percentage": usage,
            "total_active_micros": channel.active_residency(),
            "total_time_micros": channel.total_residency(),
            "states": channel.states
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if args.value_only {
        println!("{usage:.2}%");
    } else if args.summary {
        println!("Usage: {usage:>6.2}%");
    } else {
        println!("{} / {}", channel.group, channel.subgroup);
        for state in &channel.states {
            println!("  {:>6}: {:>21} µs", state.name, state.residency);
        }
        println!(
            "  {:>15}: {:>12} µs (active)",
            "→ Total active",
            channel.active_residency()
        );
        println!(
            "  {:>15}: {:>12} µs (total)",
            "→ Total",
            channel.total_residency()
        );
        println!("  {:>15}: {:>12.2} %", "→ Usage", usage);
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let report = IOReport::new(GROUP, SUBGROUP)
        .context("Failed to initialize IOReport. Are you running on an Apple Silicon Mac?")?;

    let sample1 = report
        .sample()
        .context("Failed to capture initial IOReport sample")?;
    sleep(args.time);
    let sample2 = report
        .sample()
        .context("Failed to capture second IOReport sample")?;

    let channels = IOReport::get_delta(&sample1, &sample2)
        .context("Failed to compute delta between IOReport samples")?;

    let gpu_channel = channels
        .into_iter()
        .find(|c| c.group == GROUP && c.subgroup == SUBGROUP)
        .context("No GPU performance states matched. This may occur on unsupported hardware or macOS versions.")?;

    print_channel(&gpu_channel, &args)
}

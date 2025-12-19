use anyhow::{Context, Result};
use clap::{ArgGroup, Parser};
use std::thread::sleep;
use std::time::Duration;

use siligpu::parse_duration;
use siligpu::ioreport::IOReport;

#[derive(Parser)]
#[command(
    name = "siligpu",
    about = "Apple Silicon GPU Usage Display Utility for macOS",
    version = env!("CARGO_PKG_VERSION"),
)]
#[command(group(
    ArgGroup::new("mode")
        .args(&["verbose", "summary", "value_only", "json"])
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

    /// JSON mode – output results in JSON format
    #[arg(short = 'j', long = "json")]
    json: bool,

    /// Time between samples
    /// Accepts plain numbers (ms) or units: ms, s, m, h. (e.g. `100`, `100ms`, `1s`, `1m`, `1h`).
    #[arg(short = 't', long = "time", default_value = "1000ms", value_parser = parse_duration)]
    time: Duration,
}
// `parse_duration` is provided by the library crate (see `src/lib.rs`).

fn main() -> Result<()> {
    let args = Args::parse();

    let report = IOReport::new("GPU Stats", "GPU Performance States")
        .map_err(|e| anyhow::anyhow!(e))
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

    if channels.is_empty() {
        anyhow::bail!("No GPU channels found. This tool requires an Apple Silicon Mac.");
    }

    let mut printed = false;

    for channel in channels {
        if channel.group != "GPU Stats" || channel.subgroup != "GPU Performance States" {
            continue;
        }

        printed = true;

        let usage = channel.usage();

        if args.json {
            let json_output = serde_json::json!({
                "usage_percentage": usage,
                "total_active_us": channel.active_residency(),
                "total_time_us": channel.total_residency(),
                "states": channel.states
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        } else if args.value_only {
            println!("{:.2}%", usage);
        } else if args.summary {
            println!("Usage: {:>6.2}%", usage);
        } else {
            // Verbose (default)
            println!("{:>0} / {:<0}", channel.group, channel.subgroup);
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
    }

    if !printed {
        anyhow::bail!(
            "No GPU performance states matched. This may occur on unsupported hardware or macOS versions."
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("100ms").unwrap(), Duration::from_millis(100));
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("1m").unwrap(), Duration::from_secs(60));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("500").unwrap(), Duration::from_millis(500));
        assert!(parse_duration("invalid").is_err());
    }
}

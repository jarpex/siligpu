# siligpu

> 📊 A minimal Rust-based CLI tool for measuring Apple Silicon GPU usage in snapshot form using IOReport.

`siligpu` queries the Apple Silicon GPU performance states and calculates the active usage percentage based on residency times. It's a low-level, fast, no-dependency snapshot tool for developers and power users.

---

## ✅ Features

- 🔍 One-shot snapshot of GPU residency (not a live monitor)
- 🍎 Designed for Apple Silicon Macs (M1, M2, M3, M4, M5…)
- ⏱️ Customizable sampling interval with `-t, --time` (supports ms, s, m, h)
- 📦 Uses low-level `IOReport` framework (no Metal dependency)
- 🦀 Written in Rust
- 🧩 Lightweight and fast
- 📄 JSON output support for easy parsing

---

## 📦 Installation

### Homebrew

Install via Homebrew:

```bash
brew install jarpex/formulae/siligpu
```

### From Source

```bash
git clone https://github.com/jarpex/siligpu.git
cd siligpu
cargo install --path .
```

---

## 🚀 Usage

```bash
siligpu [OPTIONS]
```

### Options

| Flag                  | Description                                                                                           |
| --------------------- | ----------------------------------------------------------------------------------------------------- |
| `-v`, `--verbose`     | Verbose mode – show detailed performance states (default)                                             |
| `-s`, `--summary`     | Summary mode – show one-line summary: `Usage: XX.XX%`                                                 |
| `-q`, `--value-only`  | Quiet mode – output only the numeric value (e.g., `12.34%`)                                           |
| `-j`, `--json`        | JSON mode – output results in JSON format                                                             |
| `-t`, `--time <TIME>` | Time between samples. Accepts plain numbers (ms) or units: `ms`, `s`, `m`, `h`. Defaults to `1000ms`. |
| `-h`, `--help`        | Print help information                                                                                |
| `-V`, `--version`     | Print version information                                                                             |

> **Time format examples:** `-t 500` (500ms), `-t 2s` (2 seconds), `--time 1m` (1 minute)

---

## 💡 Example

```bash
# Default (1 second interval, verbose)
siligpu

# 500 ms interval, summary mode
siligpu -t 500 -s

# 2-second interval, value-only
siligpu --time 2s -q

# JSON output
siligpu --json
```

### Example output (verbose)

```bash
GPU Stats  / GPU Performance States
     OFF:               993356 µs
      P1:                 6256 µs
      P2:                 2093 µs
      P3:                 3296 µs
      ...
    → Total active:      14420 µs (active)
           → Total:    1007776 µs (total)
           → Usage:       1.43 %
```

### Example output (JSON)

```json
{
  "states": [
    {
      "name": "OFF",
      "residency_micros": 993356,
      "is_active": false
    },
    {
      "name": "P1",
      "residency_micros": 6256,
      "is_active": true
    },
    {
      "name": "P2",
      "residency_micros": 2093,
      "is_active": true
    },
    {
      "name": "P3",
      "residency_micros": 3296,
      "is_active": true
    },
    {
      "name": "P4",
      "residency_micros": 2775,
      "is_active": true
    }
  ],
  "total_active_micros": 14420,
  "total_time_micros": 1007776,
  "usage_percentage": 1.43
}
```

---

## 📦 Requirements

- macOS (Big Sur 11.0 or later)
- Apple Silicon (M1, M2, M3, M4, M5...)

> On unsupported hardware (e.g., Intel Macs or older macOS versions), `siligpu` will exit with an error explaining that GPU performance states are unavailable instead of crashing.

---

## 🧪 Testing

```bash
cargo test
```

Tests cover the duration parser, GPU channel math, and error handling for invalid input.

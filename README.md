# siligpu

> 📊 A minimal Rust-based CLI tool for measuring Apple Silicon GPU usage in snapshot form using IOReport.

`siligpu` queries the Apple Silicon GPU performance states and calculates the active usage percentage based on residency times. It's a low-level, fast, no-dependency snapshot tool for developers and power users.

---

## ✅ Features

- 🔍 One-shot snapshot of GPU residency (not a live monitor)
- 🍎 Designed for Apple Silicon Macs (M1, M2, M3, M4…)
- ⏱️ Customizable sampling interval with `-t, --time` (supports ms, s, m, h)
- 📦 Uses low-level `IOReport` framework (no Metal dependency)
- 🦀 Written in Rust
- 🧩 Lightweight and fast

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
```

### Example output (verbose)

```bash
GPU Stats  / GPU Performance States
     OFF:             23840567 µs
      P1:               150146 µs
      P2:                50254 µs
      P3:                79121 µs
      ...
    → Total active:     346071 µs (active)
           → Total:   24186638 µs (total)
           → Usage:       1.43 %
```

---

## 📦 Requirements

- macOS (Big Sur 11.0 or later)
- Apple Silicon (M1, M2, M3, M4...)

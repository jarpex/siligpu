# siligpu

> **siligpu** – Apple **SILI**con **GPU** measuring utility.

A CLI snapshot tool for Apple Silicon GPU usage via IOReport. Samples GPU residency over a given interval, computes active percentage, and exits – no continuous monitoring overhead. Does not require root privileges.

Works on all Apple Silicon hardware (M-series and A-series). Tested on macOS 11 through macOS 27.

**Contents:**

- [Installation](#installation)
- [Usage](#usage)
- [Real-world usage](#real-world-usage)
- [Compatibility](#compatibility)
- [How to report issues](#how-to-report-issues)
- [Security](#security)
- [Contributing](#contributing)
- [Licenses](#licenses)

## Installation

### Homebrew

```bash
brew install jarpex/formulae/siligpu
```

### From Source

Requires a working Rust toolchain:

```bash
git clone https://github.com/jarpex/siligpu.git
cd siligpu
cargo install --path .
```

## Usage

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

> Time format examples: `-t 500` (500ms), `-t 2s` (2 seconds), `--time 1m` (1 minute)

### Usage examples

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

### Output examples

**Verbose** (default):

```bash
GPU Stats / GPU Performance States
     OFF:                776233 µs
      P1:                216182 µs
      P2:                  6208 µs
      P3:                  6502 µs
      P4:                     0 µs
      P5:                  1733 µs
      ...
   → Total active:       230625 µs (active)
          → Total:      1006858 µs (total)
          → Usage:        22.91 %
```

**Value-only** (`-q`):

```bash
22.91%
```

**Summary** (`-s`):

```bash
Usage:   1.64%
```

**JSON** (`-j`):

```json
{
  "states": [
    {"is_active": false, "name": "OFF", "residency_micros": 541002},
    {"is_active": true, "name": "P1", "residency_micros": 301833},
    ...
  ],
  "total_active_micros": 462027,
  "total_time_micros": 1003029,
  "usage_percentage": 46.06
}
```

### Real-world usage

`siligpu` integrates perfectly with system fetch tools like [fastfetch](https://github.com/fastfetch-cli/fastfetch). The `-q` (value-only) flag combined with a short sampling interval provides clean numeric output ideal for status displays.

Configuration file: `~/.config/fastfetch/config.jsonc`:

```json
{
  "$schema": "https://github.com/fastfetch-cli/fastfetch/raw/master/doc/json_schema.json",
  "modules": [
    "os",
    "kernel",
    "terminal",
    "cpu",
    "gpu",
    {
      "type": "command",
      "key": "GPU Usage",
      "keyColor": "33",
      "text": "siligpu -q -t 200ms"
    },
    "memory",
    "break",
    "colors"
  ]
}
```

Will output something like:

```
~ ❯ fastfetch
                     ..'          OS: macOS Golden Gate 27.0 (26A428) arm64
                 ,xNMM.           Kernel: Darwin 27.0.0
               .OMMMMo            Terminal: ghostty 1.3.1
               lMM"               CPU: Apple M3 Pro (5+6) @ 4.06 GHz
     .;loddo:.  .olloddol;.       GPU: Apple M3 Pro (14) @ 1.38 GHz [Integrated]
   cKMMMMMMMMMMNWMMMMMMMMMM0:     GPU Usage: 2.51%
 .KMMMMMMMMMMMMMMMMMMMMMMMWd.     Memory: 14.77 GiB / 18.00 GiB (82%)
 XMMMMMMMMMMMMMMMMMMMMMMMX.
;MMMMMMMMMMMMMMMMMMMMMMMM:
:MMMMMMMMMMMMMMMMMMMMMMMM:
.MMMMMMMMMMMMMMMMMMMMMMMMX.
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.
 'XMMMMMMMMMMMMMMMMMMMMMMMMMMk
  'XMMMMMMMMMMMMMMMMMMMMMMMMK.
    kMMMMMMMMMMMMMMMMMMMMMMd
     ;KMMMMMMMWXXWMMMMMMMk.
       "cooc*"    "*coo'"
```

## Compatibility

Literally any Apple Silicon Mac:

- macOS 11.0 (build 20A2411) or later
- Apple Silicon hardware (M-series / A-series)

> Note: On unsupported hardware or non-Apple Silicon environments, siligpu exits gracefully with an explicit error message instead of crashing.

## How to report issues

If you encounter any issues or bugs while using siligpu, please report them on the [GitHub Issues](https://github.com/jarpex/siligpu/issues) page.

## Security

Vulnerabilities should be reported privately in accordance with our [SECURITY.md](SECURITY.md) policy via [GitHub Security Advisories](https://github.com/jarpex/siligpu/security/advisories/new).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on submitting issues or pull requests.

## Licenses

See [LICENSE](LICENSE) and [THIRD_PARTY_LICENSES.html](THIRD_PARTY_LICENSES.html) for license information.

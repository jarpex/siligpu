# siligpu

> 📊 A minimal Rust-based CLI tool for measuring Apple Silicon GPU usage in snapshot form using IOReport.

`siligpu` queries the Apple Silicon GPU performance states and calculates the active usage percentage based on residency times.  
It's a low-level, fast, no-dependency snapshot tool for developers and power users.

---

## ✅ Features

- 🔍 One-shot snapshot of GPU residency (not a live monitor)
- 🍎 Designed for Apple Silicon Macs (M1, M2, M3…)
- 📦 Uses low-level `IOReport` framework (no Metal dependency)
- 🦀 Written in Rust
- 🧩 Lightweight and fast

---

## 🚀 Usage

```bash
siligpu
```

### Example output:

```bash
GPU Residency (per frequency state):

GPU Stats  / GPU Performance States    / GPUPH
              OFF:   23840567 µs
               P1:     150146 µs
               P2:      50254 µs
               P3:      79121 µs
               P4:          0 µs
               P5:      66550 µs
               P6:          0 µs
               P7:          0 µs
               P8:          0 µs
               P9:          0 µs
              P10:          0 µs
              P11:          0 µs
              P12:          0 µs
              P13:          0 µs
              P14:          0 µs
              P15:          0 µs
   → Total active:     346071 µs (active)
          → Total:   24186638 µs (total)
          → Usage:   1.43 %
```

## 📦 Requirements

- macOS
- Apple Silicon (M1, M2, M3, M4...)

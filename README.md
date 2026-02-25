# fractals-tui

Terminal fractal explorer using Unicode braille characters for sub-pixel rendering. Each terminal cell encodes an 8-dot (2x4) braille pattern, giving 8x the effective resolution of regular character-based renderers.

## Features

- **Braille sub-pixel rendering** — 2x4 dot matrix per cell for high-resolution output in any terminal
- **4 fractal types** — Mandelbrot, Julia, Burning Ship, Tricorn
- **5 color palettes** — Classic, Fire, Ocean, Neon, Grayscale with smooth interpolation
- **Mouse-driven navigation** — Click to zoom, drag to pan, scroll wheel, hold for continuous zoom
- **Autopilot mode** — Automatically finds and zooms into interesting boundary regions
- **Adaptive iterations** — Iteration depth increases automatically as you zoom deeper
- **Soft viewport bounds** — Rubber-band effect gently guides you back toward the fractal
- **Parallel computation** — Uses rayon for multi-threaded fractal calculation

## Install

```bash
cargo install --git https://github.com/ofcRS/fractals-tui
```

## Build from source

```bash
git clone https://github.com/ofcRS/fractals-tui
cd fractals-tui
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| `Arrow keys` / `WASD` | Pan |
| `+` / `-` | Zoom in / out |
| `Tab` / `Shift+Tab` | Next / previous fractal |
| `Space` | Toggle autopilot |
| `c` | Cycle color palette |
| `r` | Reset viewport |
| `[` / `]` | Decrease / increase iterations |
| `?` | Help overlay |
| `q` / `Esc` | Quit |

**Mouse**: Left-click zooms in, right-click zooms out, drag to pan, scroll to zoom at cursor. Hold click for continuous zoom.

## Requirements

- A terminal with Unicode and true-color (24-bit) support
- Rust 1.70+

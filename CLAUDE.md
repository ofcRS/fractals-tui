# fractals-tui

Terminal fractal explorer using braille characters for sub-pixel rendering.

## Build & Run

```bash
cargo build            # compile
cargo run              # run (debug)
cargo run --release    # run (optimized, recommended)
cargo clippy           # lint
```

Dependencies: ratatui (TUI framework), crossterm (terminal events), rayon (parallel computation).

## Architecture

### Rendering Pipeline

```
Viewport::generate_pixels()  →  Fractal::compute_grid()  →  render_braille()  →  FractalWidget
   (viewport.rs)                  (fractal/mod.rs)           (render/braille.rs)   (render/widget.rs)
```

1. **Viewport** (`viewport.rs`) converts terminal dimensions to complex-plane coordinates, outputting `Vec<(f64, f64)>` at braille sub-pixel resolution (`cols*2 × rows*4`)
2. **Compute** (`fractal/mod.rs`) runs `iterate()` on each point via rayon `par_iter()`, returning `Vec<IterationResult>` with iteration count, escaped flag, and smooth value
3. **Braille** (`render/braille.rs`) maps 2×4 sub-pixel blocks to Unicode braille characters (base `0x2800`), averaging smooth values for color lookup
4. **Widget** (`render/widget.rs`) implements ratatui `Widget`, writing braille chars with palette-interpolated RGB colors to the terminal buffer

### Fractal Trait

`src/fractal/mod.rs` — all fractals implement `Fractal: Send + Sync`:
- `iterate(c, max_iter) → IterationResult` — per-point escape iteration
- `default_viewport() → (re, im, zoom_w, zoom_h)` — initial view
- `compute_grid()` — default parallel dispatch via rayon (override not needed)

Implementations: `Mandelbrot`, `Julia` (fixed c = -0.7269 + 0.1889i), `BurningShip`, `Tricorn`. Factory: `all_fractals()` returns `Vec<Box<dyn Fractal>>`.

### Autopilot State Machine

`src/autopilot.rs` — cycles through 4 states:

```
Panning → ZoomingIn → Dwelling → TransitioningOut → Panning ...
```

- **Panning**: lerp toward target (speed 0.05), transitions when close or after 120 ticks
- **ZoomingIn**: exponential zoom (`*= 0.985`), transitions after 200 ticks or 1000× zoom
- **Dwelling**: pause 60 ticks (~2s at 30fps)
- **TransitioningOut**: zoom out (`*= 1.02`), then `pick_target()` selects next point

`pick_target()` samples every 4th pixel, scores by `iter_score * 0.6 + variance * 0.4` to find boundary regions.

## Key Conventions

### Braille Sub-Pixel Mapping

Each terminal cell encodes 8 dots (2 wide × 4 tall). Bit layout:

```
(0,0)=0x01  (1,0)=0x08
(0,1)=0x02  (1,1)=0x10
(0,2)=0x04  (1,2)=0x20
(0,3)=0x40  (1,3)=0x80
```

Character = `char::from_u32(0x2800 + pattern)`.

### Smooth Coloring

`smooth = iterations - ln(ln(|z|)) / ln(2)` — provides sub-iteration precision. Palette lookup normalizes via `fract(smooth / max_iter)` and linearly interpolates between anchor RGB values. 5 palettes: Classic, Fire, Ocean, Neon, Grayscale.

### Aspect Correction

Terminal chars are ~2× taller than wide. Viewport compensates: `half_h = zoom * (pixel_h / pixel_w) * 2.0`, ensuring circles render circular. Same correction applied in autopilot target coordinate conversion.

## Source Layout

```
src/
├── main.rs              # Event loop (~30fps), terminal setup, UI layout
├── app.rs               # App state, input handling, compute dispatch
├── viewport.rs          # Complex-plane ↔ pixel coordinate mapping
├── autopilot.rs         # Autopilot state machine and target selection
├── fractal/
│   ├── mod.rs           # Fractal trait, IterationResult, all_fractals()
│   ├── mandelbrot.rs    # z² + c
│   ├── julia.rs         # z² + c_const (fixed c)
│   ├── burning_ship.rs  # |Re|,|Im| variant
│   └── tricorn.rs       # conjugate variant
└── render/
    ├── mod.rs           # Module exports
    ├── braille.rs       # Sub-pixel → braille character encoding
    ├── color.rs         # Palette definitions and RGB interpolation
    └── widget.rs        # Ratatui Widget impl
```

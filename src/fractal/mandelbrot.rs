use super::{Fractal, IterationResult};

pub struct Mandelbrot;

impl Fractal for Mandelbrot {
    fn name(&self) -> &str {
        "Mandelbrot"
    }

    fn iterate(&self, c: (f64, f64), max_iter: u32) -> IterationResult {
        let (cr, ci) = c;
        let mut zr: f64 = 0.0;
        let mut zi: f64 = 0.0;
        let mut i = 0u32;

        while i < max_iter {
            let zr2 = zr * zr;
            let zi2 = zi * zi;
            if zr2 + zi2 > 4.0 {
                let smooth = i as f64 - (zr2 + zi2).ln().ln() / std::f64::consts::LN_2;
                return IterationResult {
                    iterations: i,
                    escaped: true,
                    smooth,
                };
            }
            zi = 2.0 * zr * zi + ci;
            zr = zr2 - zi2 + cr;
            i += 1;
        }

        IterationResult {
            iterations: max_iter,
            escaped: false,
            smooth: max_iter as f64,
        }
    }

    fn default_viewport(&self) -> (f64, f64, f64, f64) {
        // center_re, center_im, zoom_re, zoom_im (half-widths)
        (-0.5, 0.0, 1.5, 1.0)
    }

    fn bounds(&self) -> (f64, f64, f64, f64) {
        (-2.25, 0.75, -1.5, 1.5)
    }
}

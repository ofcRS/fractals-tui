use super::{Fractal, IterationResult};

pub struct Julia {
    pub c_re: f64,
    pub c_im: f64,
}

impl Default for Julia {
    fn default() -> Self {
        Self {
            c_re: -0.7269,
            c_im: 0.1889,
        }
    }
}

impl Fractal for Julia {
    fn name(&self) -> &str {
        "Julia"
    }

    fn iterate(&self, pixel: (f64, f64), max_iter: u32) -> IterationResult {
        let mut zr = pixel.0;
        let mut zi = pixel.1;
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
            zi = 2.0 * zr * zi + self.c_im;
            zr = zr2 - zi2 + self.c_re;
            i += 1;
        }

        IterationResult {
            iterations: max_iter,
            escaped: false,
            smooth: max_iter as f64,
        }
    }

    fn default_viewport(&self) -> (f64, f64, f64, f64) {
        (0.0, 0.0, 1.6, 1.2)
    }

    fn bounds(&self) -> (f64, f64, f64, f64) {
        (-2.0, 2.0, -1.5, 1.5)
    }
}

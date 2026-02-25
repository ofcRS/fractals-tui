pub mod mandelbrot;
pub mod julia;
pub mod burning_ship;
pub mod tricorn;

#[derive(Clone, Copy, Debug)]
pub struct IterationResult {
    pub iterations: u32,
    pub escaped: bool,
    pub smooth: f64,
}

pub trait Fractal: Send + Sync {
    fn name(&self) -> &str;
    fn iterate(&self, c: (f64, f64), max_iter: u32) -> IterationResult;
    fn default_viewport(&self) -> (f64, f64, f64, f64);

    fn bounds(&self) -> (f64, f64, f64, f64) {
        let (cr, ci, half_w, half_h) = self.default_viewport();
        (cr - half_w, cr + half_w, ci - half_h, ci + half_h)
    }

    fn compute_grid(&self, pixels: &[(f64, f64)], max_iter: u32) -> Vec<IterationResult> {
        use rayon::prelude::*;
        pixels
            .par_iter()
            .map(|&c| self.iterate(c, max_iter))
            .collect()
    }
}

pub fn all_fractals() -> Vec<Box<dyn Fractal>> {
    vec![
        Box::new(mandelbrot::Mandelbrot),
        Box::new(julia::Julia::default()),
        Box::new(burning_ship::BurningShip),
        Box::new(tricorn::Tricorn),
    ]
}

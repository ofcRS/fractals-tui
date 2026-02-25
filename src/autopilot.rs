use crate::fractal::IterationResult;
use crate::viewport::Viewport;

pub struct Autopilot {
    pub active: bool,
    target_re: f64,
    target_im: f64,
    needs_target: bool,
}

impl Autopilot {
    pub fn new() -> Self {
        Self {
            active: false,
            target_re: 0.0,
            target_im: 0.0,
            needs_target: true,
        }
    }

    pub fn toggle(&mut self, _viewport: &Viewport) {
        self.active = !self.active;
        if self.active {
            self.needs_target = true;
        }
    }

    pub fn tick(
        &mut self,
        viewport: &mut Viewport,
        results: &[IterationResult],
        pixel_w: usize,
        pixel_h: usize,
        max_iter: u32,
    ) {
        if !self.active {
            return;
        }

        if self.needs_target {
            self.pick_target(results, pixel_w, pixel_h, viewport, max_iter);
            self.needs_target = false;
        }

        // Drift toward target and zoom in
        viewport.center_re += (self.target_re - viewport.center_re) * 0.03;
        viewport.center_im += (self.target_im - viewport.center_im) * 0.03;
        viewport.zoom *= 0.993;

        // Pick a new target periodically as deeper detail emerges
        let dist = ((viewport.center_re - self.target_re).powi(2)
            + (viewport.center_im - self.target_im).powi(2))
        .sqrt();
        if dist < viewport.zoom * 0.001 {
            self.pick_target(results, pixel_w, pixel_h, viewport, max_iter);
        }
    }

    fn pick_target(
        &mut self,
        results: &[IterationResult],
        pixel_w: usize,
        pixel_h: usize,
        viewport: &Viewport,
        max_iter: u32,
    ) {
        let mut best_score = 0.0f64;
        let mut best_px = pixel_w / 2;
        let mut best_py = pixel_h / 2;

        let cell_aspect = 2.0;
        let half_w = viewport.zoom;
        let half_h = viewport.zoom * (pixel_h as f64) / (pixel_w as f64) * cell_aspect;
        let re_min = viewport.center_re - half_w;
        let im_max = viewport.center_im + half_h;
        let re_step = (2.0 * half_w) / pixel_w as f64;
        let im_step = (2.0 * half_h) / pixel_h as f64;

        let step = 4;
        for py in (step..pixel_h.saturating_sub(step)).step_by(step) {
            for px in (step..pixel_w.saturating_sub(step)).step_by(step) {
                let idx = py * pixel_w + px;
                if idx >= results.len() {
                    continue;
                }
                let r = &results[idx];

                if !r.escaped {
                    continue;
                }

                let iter_score = r.iterations as f64 / max_iter as f64;

                let mut variance = 0.0;
                let mut count = 0;
                for dy in [-(step as i32), 0, step as i32] {
                    for dx in [-(step as i32), 0, step as i32] {
                        let npx = px as i32 + dx;
                        let npy = py as i32 + dy;
                        if npx >= 0 && npx < pixel_w as i32 && npy >= 0 && npy < pixel_h as i32 {
                            let nidx = npy as usize * pixel_w + npx as usize;
                            if nidx < results.len() {
                                let diff = results[nidx].smooth - r.smooth;
                                variance += diff * diff;
                                count += 1;
                            }
                        }
                    }
                }
                if count > 0 {
                    variance /= count as f64;
                }

                let score = iter_score * 0.6 + (variance / (max_iter as f64)).sqrt() * 0.4;

                if score > best_score {
                    best_score = score;
                    best_px = px;
                    best_py = py;
                }
            }
        }

        self.target_re = re_min + best_px as f64 * re_step;
        self.target_im = im_max - best_py as f64 * im_step;
    }
}

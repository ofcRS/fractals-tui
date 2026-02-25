pub struct Viewport {
    pub center_re: f64,
    pub center_im: f64,
    pub zoom: f64,
}

impl Viewport {
    pub fn from_default(default: (f64, f64, f64, f64)) -> Self {
        let (cr, ci, half_w, _half_h) = default;
        Self {
            center_re: cr,
            center_im: ci,
            zoom: half_w,
        }
    }

    /// Generate complex-plane coordinates for every braille sub-pixel.
    /// Terminal cell = 2 wide x 4 tall braille dots.
    /// Aspect correction: terminal chars are roughly 2:1 (height:width).
    pub fn generate_pixels(&self, cols: u16, rows: u16) -> (Vec<(f64, f64)>, usize, usize) {
        let pixel_w = cols as usize * 2;
        let pixel_h = rows as usize * 4;

        // Terminal cells are ~2x taller than wide, so horizontal span needs correction
        let cell_aspect = 2.0; // approximate height:width ratio of monospace chars
        let half_w = self.zoom;
        let half_h = self.zoom * (pixel_h as f64) / (pixel_w as f64) * cell_aspect;

        let re_min = self.center_re - half_w;
        let im_max = self.center_im + half_h;
        let re_step = (2.0 * half_w) / pixel_w as f64;
        let im_step = (2.0 * half_h) / pixel_h as f64;

        let mut pixels = Vec::with_capacity(pixel_w * pixel_h);
        for py in 0..pixel_h {
            let im = im_max - py as f64 * im_step;
            for px in 0..pixel_w {
                let re = re_min + px as f64 * re_step;
                pixels.push((re, im));
            }
        }

        (pixels, pixel_w, pixel_h)
    }

    /// Convert terminal (col, row) to complex-plane coordinates.
    /// Uses the same geometry as generate_pixels() but maps a single cell center.
    pub fn screen_to_complex(&self, col: u16, row: u16, total_cols: u16, total_rows: u16) -> (f64, f64) {
        let pixel_w = total_cols as f64 * 2.0;
        let pixel_h = total_rows as f64 * 4.0;

        let cell_aspect = 2.0;
        let half_w = self.zoom;
        let half_h = self.zoom * pixel_h / pixel_w * cell_aspect;

        // Center of the braille sub-pixel block for this cell (offset by 1, 2 to hit the middle)
        let sub_x = col as f64 * 2.0 + 1.0;
        let sub_y = row as f64 * 4.0 + 2.0;

        let re = self.center_re - half_w + (sub_x / pixel_w) * 2.0 * half_w;
        let im = self.center_im + half_h - (sub_y / pixel_h) * 2.0 * half_h;

        (re, im)
    }

    pub fn pan(&mut self, delta_re: f64, delta_im: f64) {
        self.center_re += delta_re * self.zoom * 0.1;
        self.center_im += delta_im * self.zoom * 0.1;
    }

    pub fn zoom_in(&mut self, factor: f64) {
        self.zoom /= factor;
    }

    pub fn zoom_out(&mut self, factor: f64) {
        self.zoom *= factor;
    }

    pub fn reset(&mut self, default: (f64, f64, f64, f64)) {
        let (cr, ci, half_w, _half_h) = default;
        self.center_re = cr;
        self.center_im = ci;
        self.zoom = half_w;
    }

    pub fn soft_clamp(&mut self, bounds: (f64, f64, f64, f64), default_zoom: f64) {
        let (re_min, re_max, im_min, im_max) = bounds;

        // Zoom-adaptive margin: tight when zoomed out, relaxed when zoomed in
        let zoom_ratio = (self.zoom / default_zoom).clamp(0.0001, 2.0);
        let margin_factor = 0.1 + (1.0 - zoom_ratio) * 0.9;

        let bounds_cx = (re_min + re_max) / 2.0;
        let bounds_cy = (im_min + im_max) / 2.0;
        let bounds_hw = (re_max - re_min) / 2.0;
        let bounds_hh = (im_max - im_min) / 2.0;

        // Allowed center region = fractal bounds * margin_factor + current viewport half-width
        let allowed_hw = bounds_hw * margin_factor + self.zoom;
        let allowed_hh = bounds_hh * margin_factor + self.zoom;

        let target_re = self.center_re.clamp(bounds_cx - allowed_hw, bounds_cx + allowed_hw);
        let target_im = self.center_im.clamp(bounds_cy - allowed_hh, bounds_cy + allowed_hh);

        // Lerp back at 15% per frame (~10 frames to settle)
        let strength = 0.15;
        if (self.center_re - target_re).abs() > 1e-15 {
            self.center_re += (target_re - self.center_re) * strength;
        }
        if (self.center_im - target_im).abs() > 1e-15 {
            self.center_im += (target_im - self.center_im) * strength;
        }

        // Clamp zoom range: prevent zooming out to blank or in past f64 precision
        let max_zoom = default_zoom * 3.0;
        let min_zoom = 1e-13;
        self.zoom = self.zoom.clamp(min_zoom, max_zoom);
    }
}

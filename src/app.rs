use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::autopilot::Autopilot;
use crate::fractal::{self, Fractal, IterationResult};
use crate::render::color::Palette;
use crate::viewport::Viewport;

struct MouseHold {
    button: MouseButton,
    col: u16,
    row: u16,
    frame_count: u32,
}

pub struct App {
    pub fractals: Vec<Box<dyn Fractal>>,
    pub fractal_idx: usize,
    pub viewport: Viewport,
    pub max_iter: u32,
    pub palette: Palette,
    pub autopilot: Autopilot,
    pub show_help: bool,
    pub should_quit: bool,
    pub results_cache: Option<(Vec<IterationResult>, usize, usize)>,
    pub adaptive_iter: bool,
    pub dirty: bool,
    initial_zoom: f64,
    bounds: (f64, f64, f64, f64),
    canvas_cols: u16,
    canvas_rows: u16,
    drag_start: Option<(u16, u16)>,
    drag_start_center: Option<(f64, f64)>,
    last_mouse_modifiers: KeyModifiers,
    mouse_hold: Option<MouseHold>,
    is_dragging: bool,
}

impl App {
    pub fn new() -> Self {
        let fractals = fractal::all_fractals();
        let viewport = Viewport::from_default(fractals[0].default_viewport());
        let initial_zoom = viewport.zoom;
        let bounds = fractals[0].bounds();
        Self {
            fractals,
            fractal_idx: 0,
            viewport,
            max_iter: 128,
            palette: Palette::Classic,
            autopilot: Autopilot::new(),
            show_help: false,
            should_quit: false,
            results_cache: None,
            adaptive_iter: true,
            dirty: true,
            initial_zoom,
            bounds,
            canvas_cols: 0,
            canvas_rows: 0,
            drag_start: None,
            drag_start_center: None,
            last_mouse_modifiers: KeyModifiers::NONE,
            mouse_hold: None,
            is_dragging: false,
        }
    }

    pub fn current_fractal(&self) -> &dyn Fractal {
        self.fractals[self.fractal_idx].as_ref()
    }

    pub fn compute(&mut self, cols: u16, rows: u16) {
        self.canvas_cols = cols;
        self.canvas_rows = rows;
        if self.adaptive_iter {
            self.update_adaptive_iter();
        }
        let (pixels, pixel_w, pixel_h) = self.viewport.generate_pixels(cols, rows);
        let results = self.current_fractal().compute_grid(&pixels, self.max_iter);
        self.results_cache = Some((results, pixel_w, pixel_h));
    }

    fn update_adaptive_iter(&mut self) {
        let zoom_ratio = self.initial_zoom / self.viewport.zoom;
        let depth = if zoom_ratio > 1.0 {
            zoom_ratio.log2()
        } else {
            0.0
        };
        self.max_iter = (128.0 + depth * 128.0).clamp(128.0, 16384.0) as u32;
    }

    pub fn tick(&mut self) {
        // Continuous zoom from mouse hold
        if let Some(ref mut hold) = self.mouse_hold {
            hold.frame_count += 1;
        }
        let hold_info = self.mouse_hold.as_ref().map(|h| (h.button, h.col, h.row, h.frame_count));
        if let Some((button, col, row, frame_count)) = hold_info {
            if frame_count >= 4 && !self.is_dragging && self.canvas_cols > 0 && self.canvas_rows > 0
            {
                let (cursor_re, cursor_im) = self.viewport.screen_to_complex(
                    col,
                    row,
                    self.canvas_cols,
                    self.canvas_rows,
                );
                // Drift center toward cursor
                self.viewport.center_re += (cursor_re - self.viewport.center_re) * 0.15;
                self.viewport.center_im += (cursor_im - self.viewport.center_im) * 0.15;
                match button {
                    MouseButton::Left => self.viewport.zoom_in(1.03),
                    MouseButton::Right => self.viewport.zoom_out(1.03),
                    _ => {}
                }
                self.autopilot.active = false;
                self.dirty = true;
            }
        }

        // Autopilot tick
        if self.autopilot.active {
            if let Some((ref results, pixel_w, pixel_h)) = self.results_cache {
                let results_clone = results.clone();
                self.autopilot.tick(
                    &mut self.viewport,
                    &results_clone,
                    pixel_w,
                    pixel_h,
                    self.max_iter,
                );
                self.dirty = true;
            }
        }

        // Soft-clamp viewport to fractal bounds (rubber-band effect)
        let prev_re = self.viewport.center_re;
        let prev_im = self.viewport.center_im;
        let prev_zoom = self.viewport.zoom;
        self.viewport.soft_clamp(self.bounds, self.initial_zoom);
        if (self.viewport.center_re - prev_re).abs() > 1e-15
            || (self.viewport.center_im - prev_im).abs() > 1e-15
            || (self.viewport.zoom - prev_zoom).abs() > 1e-15
        {
            self.dirty = true;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.show_help {
            self.show_help = false;
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // Pan
            KeyCode::Left | KeyCode::Char('a') => {
                self.viewport.pan(-1.0, 0.0);
                self.dirty = true;
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.viewport.pan(1.0, 0.0);
                self.dirty = true;
            }
            KeyCode::Up | KeyCode::Char('w') => {
                self.viewport.pan(0.0, 1.0);
                self.dirty = true;
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.viewport.pan(0.0, -1.0);
                self.dirty = true;
            }

            // Zoom
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.viewport.zoom_in(1.2);
                self.dirty = true;
            }
            KeyCode::Char('-') => {
                self.viewport.zoom_out(1.2);
                self.dirty = true;
            }

            // Fractal cycling
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.prev_fractal();
                } else {
                    self.next_fractal();
                }
                self.dirty = true;
            }
            KeyCode::BackTab => {
                self.prev_fractal();
                self.dirty = true;
            }

            // Palette
            KeyCode::Char('c') => {
                self.palette = self.palette.next();
                self.dirty = true;
            }

            // Reset
            KeyCode::Char('r') => {
                self.viewport.reset(self.current_fractal().default_viewport());
                self.initial_zoom = self.viewport.zoom;
                self.bounds = self.current_fractal().bounds();
                self.max_iter = 128;
                self.adaptive_iter = true;
                self.dirty = true;
            }

            // Iterations (manual override disables adaptive)
            KeyCode::Char(']') => {
                self.adaptive_iter = false;
                self.max_iter = (self.max_iter + 64).min(16384);
                self.dirty = true;
            }
            KeyCode::Char('[') => {
                self.adaptive_iter = false;
                self.max_iter = self.max_iter.saturating_sub(64).max(32);
                self.dirty = true;
            }

            // Auto mode
            KeyCode::Char(' ') => {
                self.autopilot.toggle(&self.viewport);
                self.dirty = true;
            }

            // Help
            KeyCode::Char('?') => self.show_help = !self.show_help,

            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.canvas_cols == 0 || self.canvas_rows == 0 {
            return;
        }
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.last_mouse_modifiers = mouse.modifiers;
                self.drag_start = Some((mouse.column, mouse.row));
                self.drag_start_center = Some((self.viewport.center_re, self.viewport.center_im));
                self.is_dragging = false;
                self.mouse_hold = Some(MouseHold {
                    button: MouseButton::Left,
                    col: mouse.column,
                    row: mouse.row,
                    frame_count: 0,
                });
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                // If hold-zoom was active, reset drag anchors to current state
                // so pan starts cleanly from post-zoom position
                if self.mouse_hold.is_some() {
                    self.drag_start = Some((mouse.column, mouse.row));
                    self.drag_start_center =
                        Some((self.viewport.center_re, self.viewport.center_im));
                }
                self.is_dragging = true;
                self.mouse_hold = None;
                if let (Some((start_col, start_row)), Some((start_re, start_im))) =
                    (self.drag_start, self.drag_start_center)
                {
                    let pixel_w = self.canvas_cols as f64 * 2.0;
                    let pixel_h = self.canvas_rows as f64 * 4.0;
                    let cell_aspect = 2.0;
                    let half_w = self.viewport.zoom;
                    let half_h = self.viewport.zoom * pixel_h / pixel_w * cell_aspect;

                    let dx_frac = (mouse.column as f64 - start_col as f64) * 2.0 / pixel_w;
                    let dy_frac = (mouse.row as f64 - start_row as f64) * 4.0 / pixel_h;

                    self.viewport.center_re = start_re - dx_frac * 2.0 * half_w;
                    self.viewport.center_im = start_im + dy_frac * 2.0 * half_h;
                    self.autopilot.active = false;
                    self.dirty = true;
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                let held_frames = self.mouse_hold.as_ref().map_or(0, |h| h.frame_count);
                self.mouse_hold = None;

                if let Some((start_col, start_row)) = self.drag_start.take() {
                    if !self.is_dragging && held_frames < 5 {
                        // Short click: center on point, then zoom in or out
                        let (re, im) = self.viewport.screen_to_complex(
                            mouse.column,
                            mouse.row,
                            self.canvas_cols,
                            self.canvas_rows,
                        );
                        self.viewport.center_re = re;
                        self.viewport.center_im = im;
                        if self.last_mouse_modifiers.contains(KeyModifiers::CONTROL) {
                            self.viewport.zoom_out(1.5);
                        } else {
                            self.viewport.zoom_in(1.5);
                        }
                    }
                    // If held >= 5 frames, continuous zoom already happened in tick()
                    let _ = (start_col, start_row); // suppress unused warning
                    self.autopilot.active = false;
                    self.dirty = true;
                }
                self.drag_start_center = None;
                self.is_dragging = false;
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.mouse_hold = Some(MouseHold {
                    button: MouseButton::Right,
                    col: mouse.column,
                    row: mouse.row,
                    frame_count: 0,
                });
            }
            MouseEventKind::Up(MouseButton::Right) => {
                let held_frames = self.mouse_hold.as_ref().map_or(0, |h| h.frame_count);
                self.mouse_hold = None;

                if held_frames < 5 {
                    // Short right click: zoom out at cursor
                    let (re, im) = self.viewport.screen_to_complex(
                        mouse.column,
                        mouse.row,
                        self.canvas_cols,
                        self.canvas_rows,
                    );
                    self.viewport.center_re = re;
                    self.viewport.center_im = im;
                    self.viewport.zoom_out(1.5);
                    self.autopilot.active = false;
                    self.dirty = true;
                }
            }
            MouseEventKind::ScrollUp => {
                // Zoom toward cursor
                let (cursor_re, cursor_im) = self.viewport.screen_to_complex(
                    mouse.column,
                    mouse.row,
                    self.canvas_cols,
                    self.canvas_rows,
                );
                let factor = 0.15;
                self.viewport.center_re += (cursor_re - self.viewport.center_re) * factor;
                self.viewport.center_im += (cursor_im - self.viewport.center_im) * factor;
                self.viewport.zoom_in(1.15);
                self.autopilot.active = false;
                self.dirty = true;
            }
            MouseEventKind::ScrollDown => {
                let (cursor_re, cursor_im) = self.viewport.screen_to_complex(
                    mouse.column,
                    mouse.row,
                    self.canvas_cols,
                    self.canvas_rows,
                );
                let factor = 0.15;
                self.viewport.center_re += (cursor_re - self.viewport.center_re) * factor;
                self.viewport.center_im += (cursor_im - self.viewport.center_im) * factor;
                self.viewport.zoom_out(1.15);
                self.autopilot.active = false;
                self.dirty = true;
            }
            _ => {}
        }
    }

    fn next_fractal(&mut self) {
        self.fractal_idx = (self.fractal_idx + 1) % self.fractals.len();
        self.viewport
            .reset(self.current_fractal().default_viewport());
        self.initial_zoom = self.viewport.zoom;
        self.bounds = self.current_fractal().bounds();
        self.adaptive_iter = true;
    }

    fn prev_fractal(&mut self) {
        if self.fractal_idx == 0 {
            self.fractal_idx = self.fractals.len() - 1;
        } else {
            self.fractal_idx -= 1;
        }
        self.viewport
            .reset(self.current_fractal().default_viewport());
        self.initial_zoom = self.viewport.zoom;
        self.bounds = self.current_fractal().bounds();
        self.adaptive_iter = true;
    }
}

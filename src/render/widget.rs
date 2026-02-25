use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::Widget,
};

use crate::fractal::IterationResult;
use crate::render::braille::render_braille;
use crate::render::color::Palette;

pub struct FractalWidget<'a> {
    pub max_iter: u32,
    pub palette: Palette,
    pub results_cache: Option<&'a (Vec<IterationResult>, usize, usize)>,
}

impl<'a> Widget for FractalWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let (results, pixel_w, pixel_h) = if let Some(cache) = self.results_cache {
            (&cache.0, cache.1, cache.2)
        } else {
            return;
        };

        let braille_grid = render_braille(results, pixel_w, pixel_h);

        for (row_idx, row) in braille_grid.iter().enumerate() {
            if row_idx >= area.height as usize {
                break;
            }
            for (col_idx, cell) in row.iter().enumerate() {
                if col_idx >= area.width as usize {
                    break;
                }

                let x = area.x + col_idx as u16;
                let y = area.y + row_idx as u16;

                if cell.has_escaped {
                    let color = self.palette.color_for(cell.avg_smooth, self.max_iter);
                    buf[(x, y)].set_char(cell.ch).set_style(Style::new().fg(color));
                } else {
                    buf[(x, y)].set_char(' ');
                }
            }
        }
    }
}

use crate::fractal::IterationResult;

/// Braille dot offsets: each terminal cell is 2x4 dots.
/// Bit positions map to (dx, dy) within the cell:
///   Bit 0 -> (0,0)  Bit 3 -> (1,0)
///   Bit 1 -> (0,1)  Bit 4 -> (1,1)
///   Bit 2 -> (0,2)  Bit 5 -> (1,2)
///   Bit 6 -> (0,3)  Bit 7 -> (1,3)
const DOT_MAP: [(usize, usize, u8); 8] = [
    (0, 0, 0x01),
    (0, 1, 0x02),
    (0, 2, 0x04),
    (1, 0, 0x08),
    (1, 1, 0x10),
    (1, 2, 0x20),
    (0, 3, 0x40),
    (1, 3, 0x80),
];

const BRAILLE_BASE: u32 = 0x2800;

pub struct BrailleCell {
    pub ch: char,
    pub avg_smooth: f64,
    pub has_escaped: bool,
}

/// Convert a grid of IterationResults into braille characters.
/// `pixel_w` and `pixel_h` are the braille sub-pixel dimensions.
/// Returns a 2D grid of BrailleCells: cols x rows (terminal dimensions).
pub fn render_braille(
    results: &[IterationResult],
    pixel_w: usize,
    pixel_h: usize,
) -> Vec<Vec<BrailleCell>> {
    let cols = pixel_w / 2;
    let rows = pixel_h / 4;

    let mut grid = Vec::with_capacity(rows);

    for row in 0..rows {
        let mut line = Vec::with_capacity(cols);
        for col in 0..cols {
            let base_px = col * 2;
            let base_py = row * 4;

            let mut pattern: u8 = 0;
            let mut smooth_sum = 0.0;
            let mut escaped_count = 0u32;

            for &(dx, dy, bit) in &DOT_MAP {
                let px = base_px + dx;
                let py = base_py + dy;
                if px < pixel_w && py < pixel_h {
                    let idx = py * pixel_w + px;
                    let result = &results[idx];
                    if result.escaped {
                        pattern |= bit;
                        smooth_sum += result.smooth;
                        escaped_count += 1;
                    }
                }
            }

            let avg_smooth = if escaped_count > 0 {
                smooth_sum / escaped_count as f64
            } else {
                0.0
            };

            let ch = char::from_u32(BRAILLE_BASE + pattern as u32).unwrap_or(' ');

            line.push(BrailleCell {
                ch,
                avg_smooth,
                has_escaped: escaped_count > 0,
            });
        }
        grid.push(line);
    }

    grid
}

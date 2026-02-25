use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    Classic,
    Fire,
    Ocean,
    Neon,
    Grayscale,
}

impl Palette {
    pub const ALL: &[Palette] = &[
        Palette::Classic,
        Palette::Fire,
        Palette::Ocean,
        Palette::Neon,
        Palette::Grayscale,
    ];

    pub fn name(&self) -> &str {
        match self {
            Palette::Classic => "Classic",
            Palette::Fire => "Fire",
            Palette::Ocean => "Ocean",
            Palette::Neon => "Neon",
            Palette::Grayscale => "Grayscale",
        }
    }

    pub fn next(&self) -> Palette {
        let idx = Palette::ALL.iter().position(|p| p == self).unwrap_or(0);
        Palette::ALL[(idx + 1) % Palette::ALL.len()]
    }

    fn anchors(&self) -> &[(f64, u8, u8, u8)] {
        match self {
            Palette::Classic => &[
                (0.0, 0, 7, 100),
                (0.16, 32, 107, 203),
                (0.42, 237, 255, 255),
                (0.6425, 255, 170, 0),
                (0.8575, 0, 2, 0),
                (1.0, 0, 7, 100),
            ],
            Palette::Fire => &[
                (0.0, 0, 0, 0),
                (0.25, 128, 0, 0),
                (0.5, 255, 128, 0),
                (0.75, 255, 255, 0),
                (1.0, 255, 255, 255),
            ],
            Palette::Ocean => &[
                (0.0, 0, 0, 32),
                (0.25, 0, 64, 128),
                (0.5, 0, 180, 220),
                (0.75, 100, 220, 255),
                (1.0, 200, 255, 255),
            ],
            Palette::Neon => &[
                (0.0, 0, 0, 0),
                (0.2, 255, 0, 128),
                (0.4, 0, 255, 255),
                (0.6, 255, 255, 0),
                (0.8, 128, 0, 255),
                (1.0, 0, 0, 0),
            ],
            Palette::Grayscale => &[
                (0.0, 0, 0, 0),
                (0.5, 180, 180, 180),
                (1.0, 255, 255, 255),
            ],
        }
    }

    pub fn color_for(&self, smooth: f64, max_iter: u32) -> Color {
        let normalized = (smooth / max_iter as f64).fract();
        let anchors = self.anchors();

        // Find the two anchors to interpolate between
        let mut i = 0;
        while i < anchors.len() - 1 && anchors[i + 1].0 < normalized {
            i += 1;
        }
        if i >= anchors.len() - 1 {
            let a = anchors[anchors.len() - 1];
            return Color::Rgb(a.1, a.2, a.3);
        }

        let (t0, r0, g0, b0) = anchors[i];
        let (t1, r1, g1, b1) = anchors[i + 1];

        let t = if (t1 - t0).abs() < 1e-10 {
            0.0
        } else {
            (normalized - t0) / (t1 - t0)
        };

        let r = lerp(r0, r1, t);
        let g = lerp(g0, g1, t);
        let b = lerp(b0, b1, t);

        Color::Rgb(r, g, b)
    }
}

fn lerp(a: u8, b: u8, t: f64) -> u8 {
    let result = a as f64 + (b as f64 - a as f64) * t;
    result.clamp(0.0, 255.0) as u8
}

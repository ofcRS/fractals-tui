mod app;
mod autopilot;
mod fractal;
mod render;
mod viewport;

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    DefaultTerminal, Frame,
};

use app::App;
use render::widget::FractalWidget;

const TICK_RATE: Duration = Duration::from_millis(33); // ~30fps

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    io::stdout().execute(EnableMouseCapture)?;

    let mut terminal = ratatui::init();
    let result = run(&mut terminal);

    ratatui::restore();
    io::stdout().execute(DisableMouseCapture)?;

    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        let frame_start = Instant::now();

        // Drain all queued events before computing
        let timeout = TICK_RATE.saturating_sub(frame_start.elapsed());
        if event::poll(timeout)? {
            loop {
                match event::read()? {
                    Event::Key(key) => app.handle_key(key),
                    Event::Mouse(mouse) => app.handle_mouse(mouse),
                    Event::Resize(_, _) => {
                        app.dirty = true;
                    }
                    _ => {}
                }
                // Drain remaining events without waiting
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }

        // Autopilot tick
        app.tick();

        // Compute fractal only when dirty
        let size = terminal.size()?;
        let canvas_height = size.height.saturating_sub(1); // reserve 1 row for status
        if app.dirty && canvas_height > 0 && size.width > 0 {
            app.compute(size.width, canvas_height);
            app.dirty = false;
        }

        // Render
        terminal.draw(|frame| ui(frame, &app))?;
    }
}

fn ui(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Min(1),    // fractal canvas
        Constraint::Length(1), // status bar
    ])
    .split(frame.area());

    // Fractal canvas
    let widget = FractalWidget {
        max_iter: app.max_iter,
        palette: app.palette,
        results_cache: app.results_cache.as_ref(),
    };
    frame.render_widget(widget, chunks[0]);

    // Status bar
    let mode = if app.autopilot.active {
        "AUTO".to_string()
    } else {
        "MANUAL".to_string()
    };

    let zoom_display = if app.viewport.zoom < 0.001 {
        format!("{:.2e}", app.viewport.zoom)
    } else {
        format!("{:.6}", app.viewport.zoom)
    };

    let status = Line::from(vec![
        Span::styled(
            format!(" {} ", app.current_fractal().name()),
            Style::new().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" "),
        Span::styled(
            format!("Zoom: {}", zoom_display),
            Style::new().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!(
                "Iter: {}{}",
                app.max_iter,
                if app.adaptive_iter { "A" } else { "" }
            ),
            Style::new().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(mode, Style::new().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled(
            format!("Palette: {}", app.palette.name()),
            Style::new().fg(Color::Cyan),
        ),
        Span::raw(" | "),
        Span::styled("? help", Style::new().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(status), chunks[1]);

    // Help overlay
    if app.show_help {
        render_help(frame);
    }
}

fn render_help(frame: &mut Frame) {
    let area = frame.area();
    let help_w = 42u16;
    let help_h = 23u16;
    let x = area.width.saturating_sub(help_w) / 2;
    let y = area.height.saturating_sub(help_h) / 2;
    let help_area = Rect::new(x, y, help_w.min(area.width), help_h.min(area.height));

    frame.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(Span::styled(
            "  Fractal TUI - Controls",
            Style::new().fg(Color::Cyan).bold(),
        )),
        Line::from(""),
        Line::from("  Arrow/WASD    Pan"),
        Line::from("  +/-           Zoom in/out"),
        Line::from("  Tab/S-Tab     Next/prev fractal"),
        Line::from("  Space         Toggle auto mode"),
        Line::from("  c             Cycle palette"),
        Line::from("  r             Reset viewport"),
        Line::from("  [ / ]         Decrease/increase iters"),
        Line::from("  ?             Toggle this help"),
        Line::from("  q / Esc       Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Mouse",
            Style::new().fg(Color::Cyan).bold(),
        )),
        Line::from("  L-Click       Zoom in at cursor"),
        Line::from("  L-Hold        Continuous zoom in"),
        Line::from("  R-Click       Zoom out at cursor"),
        Line::from("  R-Hold        Continuous zoom out"),
        Line::from("  Drag          Pan"),
        Line::from("  Scroll        Zoom at cursor"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press any key to close",
            Style::new().fg(Color::DarkGray),
        )),
    ];

    let block = Block::bordered()
        .border_style(Style::new().fg(Color::Cyan))
        .style(Style::new().bg(Color::Black));
    let paragraph = Paragraph::new(help_text).block(block);
    frame.render_widget(paragraph, help_area);
}

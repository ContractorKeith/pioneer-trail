//! Render a `.px` sprite in the terminal using the game's renderer.

use std::{
    fs,
    io::{self, stdout},
};

use anyhow::Context;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pioneer_trail::art::{render, ColorMode, PxImage};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

#[derive(Parser, Debug)]
#[command(name = "px-preview", about = "Preview a Pioneer Trail .px sprite")]
struct Args {
    /// Path to a `.px` file.
    path: std::path::PathBuf,
    /// Palette: truecolor, 256, 16, or mono.
    #[arg(long, default_value = "truecolor")]
    color: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mode = match args.color.as_str() {
        "truecolor" => ColorMode::TrueColor,
        "256" => ColorMode::Ansi256,
        "16" => ColorMode::Ansi16,
        "mono" => ColorMode::Mono,
        _ => anyhow::bail!("--color must be truecolor, 256, 16, or mono"),
    };
    let source = fs::read_to_string(&args.path)
        .with_context(|| format!("reading {}", args.path.display()))?;
    let image =
        PxImage::parse(&source).with_context(|| format!("parsing {}", args.path.display()))?;
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(output))?;
    terminal.draw(|frame| {
        render(
            &image,
            frame.buffer_mut(),
            Rect::new(0, 0, image.width(), image.cell_height()),
            mode,
        )
    })?;
    loop {
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            break;
        }
    }
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

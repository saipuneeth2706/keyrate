mod app;
mod input;
mod theme;
mod view;
mod words;

use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use app::App;

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("keyrate {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!(
            "keyrate {} — a typing practice TUI for the terminal

USAGE:
    keyrate [OPTIONS]

OPTIONS:
    -v, --version    Print version
    -h, --help       Print this help",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|frame| app.view(frame))?;

        if event::poll(Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            app.handle_key(key);
        }

        app.check_time_limit();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

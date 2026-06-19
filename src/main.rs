mod app;
mod diff;
mod field;
mod git;
mod input;
mod status;
mod theme;
mod tree;
mod ui;

use app::App;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    while !app.should_quit() {
        terminal.draw(|f| ui::draw(f, &app))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.perform(input::map(&app, key));
        }
    }
    Ok(())
}

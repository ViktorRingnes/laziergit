mod app;
mod diff;
mod field;
mod git;
mod input;
mod layout;
mod mouse;
mod nav;
mod status;
mod theme;
mod tree;
mod ui;

use app::App;
use color_eyre::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::io::stdout;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture)?;
    let result = run(&mut terminal);
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let mut app = App::new();
    while !app.should_quit() {
        let area = terminal.draw(|f| ui::draw(f, &app))?.area;
        app.set_area(area);
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                app.perform(input::map(&app, key));
            }
            Event::Mouse(mouse) => app.perform(mouse::map(&app, mouse)),
            _ => {}
        }
    }
    Ok(())
}

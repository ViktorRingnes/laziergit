mod app;
mod diff;
mod field;
mod git;
mod input;
mod layout;
mod mouse;
mod nav;
mod status;
mod task;
mod theme;
mod tree;
mod ui;

use app::App;
use color_eyre::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::execute;
use ratatui::DefaultTerminal;
use std::io::stdout;
use std::time::Duration;

const IDLE_TICK: Duration = Duration::from_millis(1000);
const BUSY_TICK: Duration = Duration::from_millis(80);

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
        let timeout = if app.busy() { BUSY_TICK } else { IDLE_TICK };
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    app.perform(input::map(&app, key));
                }
                Event::Mouse(mouse) => app.perform(mouse::map(&app, mouse)),
                _ => {}
            }
        }
        app.tick();
    }
    Ok(())
}

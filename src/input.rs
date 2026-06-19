use crate::app::{Action, App, Focus, Mode};
use crate::field::EditOp;
use crossterm::event::{KeyCode, KeyEvent};

pub fn map(app: &App, key: KeyEvent) -> Action {
    match &app.mode {
        Mode::Normal => normal(app.focus, key.code),
        Mode::Commit(_) | Mode::Branch(_) => editing(key.code),
        Mode::Confirm => confirm(key.code),
        Mode::Help => Action::Cancel,
    }
}

fn normal(focus: Focus, code: KeyCode) -> Action {
    use KeyCode::*;
    match focus {
        Focus::Sidebar => match code {
            Char('q') => Action::Quit,
            Char('j') | Down => Action::NavNext,
            Char('k') | Up => Action::NavPrev,
            Char('h') | Left => Action::Collapse,
            Char('l') | Right => Action::Expand,
            Enter => Action::EnterRow,
            Tab => Action::ToggleFocus,
            Char(' ') => Action::StageToggle,
            Char('c') => Action::OpenCommit,
            Char('b') => Action::OpenBranch,
            Char('p') => Action::Pull,
            Char('P') => Action::Push,
            Char('X') => Action::OpenConfirm,
            Char('?') => Action::OpenHelp,
            _ => Action::Noop,
        },
        Focus::Diff => match code {
            Char('q') => Action::Quit,
            Char('j') | Down => Action::ScrollDiff(1),
            Char('k') | Up => Action::ScrollDiff(-1),
            Char('?') => Action::OpenHelp,
            Enter | Tab => Action::ToggleFocus,
            _ => Action::Noop,
        },
    }
}

fn editing(code: KeyCode) -> Action {
    match code {
        KeyCode::Char(c) => Action::Edit(EditOp::Insert(c)),
        KeyCode::Backspace => Action::Edit(EditOp::Backspace),
        KeyCode::Left => Action::Edit(EditOp::Left),
        KeyCode::Right => Action::Edit(EditOp::Right),
        KeyCode::Enter => Action::Submit,
        KeyCode::Esc => Action::Cancel,
        _ => Action::Noop,
    }
}

fn confirm(code: KeyCode) -> Action {
    match code {
        KeyCode::Char('y') | KeyCode::Enter => Action::Submit,
        KeyCode::Char('n') | KeyCode::Esc => Action::Cancel,
        _ => Action::Noop,
    }
}

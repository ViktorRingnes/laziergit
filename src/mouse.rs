use crate::app::Action;
use crate::app::App;
use crate::layout;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub fn map(app: &App, event: MouseEvent) -> Action {
    let regions = layout::regions(app.area);
    let (column, row) = (event.column, event.row);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if layout::contains(regions.sidebar_inner, column, row) {
                match row_at(app, regions.sidebar_inner.y, row) {
                    Some(index) => Action::SelectRow(index),
                    None => Action::FocusSidebar,
                }
            } else if layout::contains(regions.diff, column, row) {
                Action::FocusDiff
            } else {
                Action::Noop
            }
        }
        MouseEventKind::ScrollDown => wheel(app, &regions, column, row, 1),
        MouseEventKind::ScrollUp => wheel(app, &regions, column, row, -1),
        _ => Action::Noop,
    }
}

fn wheel(app: &App, regions: &layout::Regions, column: u16, row: u16, direction: i32) -> Action {
    if layout::contains(regions.diff, column, row) {
        Action::ScrollDiff(direction * app.wheel_step())
    } else if layout::contains(regions.sidebar, column, row) {
        if direction > 0 {
            Action::NavNext
        } else {
            Action::NavPrev
        }
    } else {
        Action::Noop
    }
}

fn row_at(app: &App, top: u16, row: u16) -> Option<usize> {
    let offset = row.checked_sub(top)? as usize;
    let index = app.sidebar_offset + offset;
    (index < app.rows.len()).then_some(index)
}

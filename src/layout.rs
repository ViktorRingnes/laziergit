use ratatui::layout::{Constraint, Layout, Rect};

pub struct Regions {
    pub header: Rect,
    pub sidebar: Rect,
    pub sidebar_inner: Rect,
    pub diff: Rect,
    pub diff_inner: Rect,
    pub footer: Rect,
}

pub fn regions(area: Rect) -> Regions {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    let [sidebar, diff] =
        Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)]).areas(body);
    Regions {
        header,
        sidebar,
        sidebar_inner: inner(sidebar),
        diff,
        diff_inner: inner(diff),
        footer,
    }
}

pub fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x + area.width
        && row >= area.y
        && row < area.y + area.height
}

fn inner(area: Rect) -> Rect {
    Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    }
}

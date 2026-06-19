use crate::app::App;
use crate::theme;
use crate::tree::{Row, RowKind};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{HighlightSpacing, List, ListItem, ListState},
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == crate::app::Focus::Sidebar;
    let items: Vec<ListItem> = build_items(app);
    let list = List::new(items)
        .block(super::panel("files", focused))
        .highlight_style(Style::new().bg(theme::SELECTION_BG))
        .highlight_symbol("▎ ")
        .highlight_spacing(HighlightSpacing::Always);
    let selected = (!app.rows.is_empty()).then_some(app.selected);
    let mut state = ListState::default().with_selected(selected);
    f.render_stateful_widget(list, area, &mut state);
}

fn build_items(app: &App) -> Vec<ListItem<'static>> {
    let guides = app.theme.guides();
    let mut last_at: Vec<bool> = Vec::new();
    let mut items = Vec::new();

    for row in &app.rows {
        let depth = row.depth as usize;
        last_at.truncate(depth);
        last_at.push(row.is_last);

        let mut spans: Vec<Span> = Vec::new();
        for (column, &ended) in last_at.iter().take(depth).enumerate() {
            let glyph = if column == depth - 1 {
                if row.is_last {
                    guides.last
                } else {
                    guides.branch
                }
            } else if ended {
                guides.blank
            } else {
                guides.vertical
            };
            spans.push(Span::styled(
                glyph.to_string(),
                Style::new().fg(theme::GUIDE),
            ));
        }

        match row.kind {
            RowKind::Dir => push_dir(app, row, &mut spans),
            RowKind::File { index } => push_file(app, row, index, &mut spans),
        }
        items.push(ListItem::new(Line::from(spans)));
    }
    items
}

fn push_dir(app: &App, row: &Row, spans: &mut Vec<Span<'static>>) {
    spans.push(Span::styled(
        format!("{} ", theme::folder_glyph(row.expanded)),
        Style::new().fg(theme::FOLDER),
    ));
    if app.theme.nerd_fonts {
        spans.push(Span::styled(
            format!("{} ", theme::folder_icon(row.expanded)),
            Style::new().fg(theme::FOLDER),
        ));
    }
    spans.push(Span::styled(
        row.name.clone(),
        Style::new().fg(folder_color(app, &row.path)),
    ));
}

fn push_file(app: &App, row: &Row, index: usize, spans: &mut Vec<Span<'static>>) {
    if app.theme.nerd_fonts {
        let (icon, color) = theme::icon_for(&row.path);
        spans.push(Span::styled(format!("{icon} "), Style::new().fg(color)));
    }
    let file = &app.status.files[index];
    spans.push(Span::styled(row.name.clone(), Style::new().fg(theme::TEXT)));
    spans.push(Span::raw(" "));
    spans.push(Span::styled(
        file.kind.letter().to_string(),
        Style::new().fg(theme::status_color(file)),
    ));
}

fn folder_color(app: &App, path: &str) -> Color {
    let prefix = format!("{path}/");
    let (mut staged, mut unstaged) = (false, false);
    for file in &app.status.files {
        if file.path == path || file.path.starts_with(&prefix) {
            staged |= file.staged;
            unstaged |= file.unstaged;
        }
    }
    match (staged, unstaged) {
        (true, true) => theme::YELLOW,
        (true, false) => theme::GREEN,
        (false, true) => theme::RED,
        (false, false) => theme::FOLDER,
    }
}

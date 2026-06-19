use crate::app::App;
use crate::diff::{Cell, Kind};
use crate::theme;
use crate::tree::RowKind;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Paragraph,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == crate::app::Focus::Diff;
    let block = super::panel(&title(app), focused);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.diff_rows.is_empty() {
        placeholder(f, inner, "no changes");
        return;
    }

    let [lcol, divcol, rcol] = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Length(1),
        Constraint::Percentage(50),
    ])
    .areas(inner);

    let rows = &app.diff_rows;
    let gutter = crate::diff::gutter_width(rows);
    let height = inner.height as usize;
    let start = (app.diff_scroll as usize).min(rows.len().saturating_sub(height));
    let buf = f.buffer_mut();

    for (offset, row) in rows[start..(start + height).min(rows.len())]
        .iter()
        .enumerate()
    {
        let y = inner.y + offset as u16;
        if row.left.kind == Kind::File {
            buf.set_style(line_rect(inner, y), Style::new().bg(theme::SURFACE));
            let label = if app.theme.nerd_fonts {
                let (icon, _) = theme::icon_for(&row.left.text);
                format!("  {icon}  {} ", row.left.text)
            } else {
                format!("  {} ", row.left.text)
            };
            buf.set_stringn(
                inner.x,
                y,
                label,
                inner.width as usize,
                Style::new()
                    .fg(theme::ACCENT)
                    .bg(theme::SURFACE)
                    .add_modifier(Modifier::BOLD),
            );
            continue;
        }
        if row.left.kind == Kind::Hunk {
            buf.set_style(line_rect(inner, y), Style::new().bg(theme::HUNK_BG));
            let label = if row.left.text.is_empty() {
                String::new()
            } else {
                format!("  {} ", row.left.text)
            };
            buf.set_stringn(
                inner.x,
                y,
                label,
                inner.width as usize,
                Style::new().fg(theme::HUNK_FG).bg(theme::HUNK_BG),
            );
            continue;
        }
        paint_cell(buf, lcol, y, &row.left, gutter);
        paint_cell(buf, rcol, y, &row.right, gutter);
        buf.set_stringn(
            divcol.x,
            y,
            "│",
            1,
            Style::new().fg(theme::BORDER).bg(theme::BASE),
        );
    }
}

fn paint_cell(buf: &mut Buffer, col: Rect, y: u16, cell: &Cell, gutter: usize) {
    let (bg, fg, gutter_fg) = palette(cell.kind);
    buf.set_style(line_rect(col, y), Style::new().bg(bg));
    if cell.kind == Kind::Empty {
        return;
    }
    let num = cell
        .num
        .map(|n| format!("{n:>gutter$}"))
        .unwrap_or_else(|| " ".repeat(gutter));
    let mut x = buf
        .set_stringn(col.x, y, num, gutter, Style::new().fg(gutter_fg).bg(bg))
        .0;
    x = buf
        .set_stringn(x, y, "│", 1, Style::new().fg(theme::GUTTER_SEP_FG).bg(bg))
        .0;
    x = buf.set_stringn(x, y, " ", 1, Style::new().bg(bg)).0;
    let avail = (col.x + col.width).saturating_sub(x) as usize;
    buf.set_stringn(
        x,
        y,
        truncate(&cell.text, avail),
        avail,
        Style::new().fg(fg).bg(bg),
    );
}

fn palette(kind: Kind) -> (Color, Color, Color) {
    match kind {
        Kind::Context => (theme::BASE, theme::CONTEXT_FG, theme::GUTTER_FG),
        Kind::Del => (theme::DEL_BG, theme::DEL_FG, theme::DEL_GUTTER_FG),
        Kind::Add => (theme::ADD_BG, theme::ADD_FG, theme::ADD_GUTTER_FG),
        Kind::Empty => (theme::FILLER_BG, theme::CONTEXT_FG, theme::GUTTER_FG),
        Kind::Hunk | Kind::File => (theme::HUNK_BG, theme::HUNK_FG, theme::HUNK_FG),
    }
}

fn truncate(text: &str, avail: usize) -> String {
    if avail == 0 {
        return String::new();
    }
    if text.chars().count() <= avail {
        return text.to_string();
    }
    let mut out: String = text.chars().take(avail.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn line_rect(area: Rect, y: u16) -> Rect {
    Rect {
        x: area.x,
        y,
        width: area.width,
        height: 1,
    }
}

fn title(app: &App) -> String {
    match app
        .rows
        .get(app.selected)
        .map(|r| (r.kind, r.path.as_str()))
    {
        Some((RowKind::File { .. }, path)) => path.to_string(),
        Some((RowKind::Dir, path)) => {
            let prefix = format!("{path}/");
            let count = app
                .status
                .files
                .iter()
                .filter(|f| f.path.starts_with(&prefix))
                .count();
            format!("{path}/  ({count})")
        }
        None => "diff".to_string(),
    }
}

fn placeholder(f: &mut Frame, area: Rect, msg: &str) {
    f.render_widget(
        Paragraph::new(Line::styled(msg, Style::new().fg(theme::OVERLAY)))
            .alignment(Alignment::Center)
            .style(Style::new().bg(theme::BASE)),
        area,
    );
}

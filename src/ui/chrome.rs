use crate::app::App;
use crate::theme;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

const CHIPS: &[(&str, &str)] = &[
    ("j/k", "move"),
    ("space", "stage"),
    ("enter", "open"),
    ("c", "commit"),
    ("p", "pull"),
    ("P", "push"),
    ("b", "branch"),
    ("?", "help"),
];

pub fn header(f: &mut Frame, area: Rect, app: &App) {
    let branch = &app.status.branch;
    let mut spans = vec![Span::styled(
        format!(" {} ", branch.name),
        Style::new()
            .fg(theme::BASE)
            .bg(theme::ACCENT)
            .add_modifier(Modifier::BOLD),
    )];
    if branch.ahead > 0 || branch.behind > 0 {
        spans.push(Span::styled(
            format!("  ↑{} ↓{}", branch.ahead, branch.behind),
            Style::new().fg(theme::MUTED),
        ));
    }
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::new().bg(theme::HEADER_BG)),
        area,
    );
}

pub fn footer(f: &mut Frame, area: Rect, app: &App) {
    let bg = Style::new().bg(theme::HEADER_BG);
    if let Some(pending) = app.pending_label() {
        f.render_widget(
            Paragraph::new(Line::styled(
                format!(" {pending}"),
                Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
            ))
            .style(bg),
            area,
        );
        return;
    }
    if !app.message.is_empty() {
        f.render_widget(
            Paragraph::new(Line::styled(
                format!(" {}", app.message),
                Style::new().fg(theme::TEXT),
            ))
            .style(bg),
            area,
        );
        return;
    }
    let chip = Style::new()
        .fg(theme::ACCENT)
        .bg(theme::SURFACE)
        .add_modifier(Modifier::BOLD);
    let mut spans = vec![Span::raw(" ")];
    for (key, label) in CHIPS {
        spans.push(Span::styled(format!(" {key} "), chip));
        spans.push(Span::styled(
            format!(" {label}  "),
            Style::new().fg(theme::MUTED),
        ));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(bg), area);
}

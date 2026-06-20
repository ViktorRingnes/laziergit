mod chrome;
mod diff;
mod sidebar;

use crate::app::{App, Mode};
use crate::field::Input;
use crate::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

pub fn draw(f: &mut Frame, app: &App) {
    let r = crate::layout::regions(f.area());

    chrome::header(f, r.header, app);
    sidebar::render(f, r.sidebar, app);
    diff::render(f, r.diff, app);
    chrome::footer(f, r.footer, app);

    match &app.mode {
        Mode::Commit(input) => input_popup(f, "commit message", input, None),
        Mode::Branch(input) => {
            input_popup(f, "switch or create branch", input, Some(&app.status.branch.name))
        }
        Mode::Confirm => confirm_popup(f),
        Mode::Help => help_popup(f),
        Mode::Normal => {}
    }
}

fn help_popup(f: &mut Frame) {
    let area = popup(f.area(), 64, 16);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(Line::styled(
            " keybindings ",
            Style::new().fg(theme::ACCENT),
        ))
        .padding(Padding::new(2, 2, 1, 1))
        .style(Style::new().bg(theme::BASE));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(inner);
    f.render_widget(Paragraph::new(help_left()), left);
    f.render_widget(Paragraph::new(help_right()), right);
}

fn help_left() -> Text<'static> {
    Text::from(vec![
        help_header("navigate"),
        help_entry("j / k", "move up / down"),
        help_entry("h / l", "fold / unfold"),
        help_entry("enter", "open / fold"),
        help_entry("tab", "switch pane"),
        Line::raw(""),
        help_header("remote"),
        help_entry("p", "pull"),
        help_entry("P", "push"),
    ])
}

fn help_right() -> Text<'static> {
    Text::from(vec![
        help_header("stage & commit"),
        help_entry("space", "stage / unstage"),
        help_entry("c", "commit"),
        help_entry("X", "discard all"),
        Line::raw(""),
        help_header("branch"),
        help_entry("b", "switch / create"),
        Line::raw(""),
        help_header("general"),
        help_entry("q", "quit"),
        help_entry("? / esc", "close"),
    ])
}

fn help_header(text: &str) -> Line<'static> {
    Line::styled(
        text.to_string(),
        Style::new().fg(theme::ACCENT).add_modifier(Modifier::BOLD),
    )
}

fn help_entry(key: &str, label: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {key:>7} "),
            Style::new().fg(theme::TEXT).bg(theme::SURFACE),
        ),
        Span::styled(format!("  {label}"), Style::new().fg(theme::MUTED)),
    ])
}

pub(super) fn border_style(active: bool) -> Style {
    if active {
        Style::new().fg(theme::BORDER_FOCUS)
    } else {
        Style::new().fg(theme::BORDER)
    }
}

pub(super) fn panel(title: &str, active: bool) -> Block<'static> {
    let title_color = if active { theme::ACCENT } else { theme::MUTED };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(border_style(active))
        .title(Line::styled(
            format!(" {title} "),
            Style::new().fg(title_color),
        ))
        .style(Style::new().bg(theme::BASE))
}

fn input_popup(f: &mut Frame, title: &str, input: &Input, hint: Option<&str>) {
    let height = if hint.is_some() { 4 } else { 3 };
    let area = popup(f.area(), 60, height);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::ACCENT))
        .title(Line::styled(
            format!(" {title} "),
            Style::new().fg(theme::ACCENT),
        ))
        .style(Style::new().bg(theme::BASE));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let field = Rect { height: 1, ..inner };
    f.render_widget(
        Paragraph::new(input.value.as_str()).style(Style::new().fg(theme::TEXT)),
        field,
    );
    f.set_cursor_position((field.x + input.cursor_col(), field.y));

    if let Some(branch) = hint {
        let below = Rect {
            y: inner.y + 1,
            height: inner.height.saturating_sub(1),
            ..inner
        };
        f.render_widget(
            Paragraph::new(format!("on {branch}")).style(Style::new().fg(theme::MUTED)),
            below,
        );
    }
}

fn confirm_popup(f: &mut Frame) {
    let area = popup(f.area(), 60, 5);
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme::RED))
        .title(Line::styled(
            " discard everything? ",
            Style::new().fg(theme::RED),
        ))
        .style(Style::new().bg(theme::BASE));
    let inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    let body = Text::from(vec![
        Line::styled(
            "git reset --hard && git clean -fd",
            Style::new().fg(theme::MUTED),
        ),
        Line::raw(""),
        Line::styled("[y] yes    [n] no", Style::new().fg(theme::YELLOW)),
    ]);
    f.render_widget(Paragraph::new(body), inner);
}

fn popup(area: Rect, percent_width: u16, height: u16) -> Rect {
    let [centered] = Layout::horizontal([Constraint::Percentage(percent_width)])
        .flex(Flex::Center)
        .areas(area);
    let [centered] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(centered);
    centered
}

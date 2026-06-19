use crate::diff::{self, DiffRow};
use crate::field::{EditOp, Input};
use crate::git;
use crate::status::{self, FileEntry, RepoStatus};
use crate::theme::Theme;
use crate::tree::{self, Row, RowKind, TreeNode};
use color_eyre::Result;
use std::collections::HashSet;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Diff,
}

pub enum Mode {
    Normal,
    Commit(Input),
    Branch(Input),
    Confirm,
    Help,
}

pub enum Action {
    Noop,
    Quit,
    NavNext,
    NavPrev,
    Collapse,
    Expand,
    EnterRow,
    ToggleFocus,
    ScrollDiff(i32),
    StageToggle,
    OpenCommit,
    OpenBranch,
    OpenConfirm,
    OpenHelp,
    Edit(EditOp),
    Submit,
    Cancel,
    Pull,
    Push,
}

pub struct App {
    pub status: RepoStatus,
    pub tree: Vec<TreeNode>,
    pub rows: Vec<Row>,
    pub collapsed: HashSet<usize>,
    pub selected: usize,
    pub focus: Focus,
    pub diff_rows: Vec<DiffRow>,
    pub diff_scroll: u16,
    pub mode: Mode,
    pub message: String,
    pub theme: Theme,
    quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = App {
            status: RepoStatus::default(),
            tree: Vec::new(),
            rows: Vec::new(),
            collapsed: HashSet::new(),
            selected: 0,
            focus: Focus::Sidebar,
            diff_rows: Vec::new(),
            diff_scroll: 0,
            mode: Mode::Normal,
            message: String::new(),
            theme: Theme::detect(),
            quit: false,
        };
        app.refresh();
        app
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::Noop => {}
            Action::Quit => self.quit = true,
            Action::NavNext => self.move_selection(1),
            Action::NavPrev => self.move_selection(-1),
            Action::Collapse => self.collapse(),
            Action::Expand => self.expand(),
            Action::EnterRow => self.enter_row(),
            Action::ToggleFocus => self.toggle_focus(),
            Action::ScrollDiff(delta) => {
                self.diff_scroll = (self.diff_scroll as i32 + delta).max(0) as u16
            }
            Action::StageToggle => self.stage_toggle(),
            Action::OpenCommit => self.mode = Mode::Commit(Input::default()),
            Action::OpenBranch => self.mode = Mode::Branch(Input::default()),
            Action::OpenConfirm => self.mode = Mode::Confirm,
            Action::OpenHelp => self.mode = Mode::Help,
            Action::Edit(op) => {
                if let Some(input) = self.input_mut() {
                    input.apply(op);
                }
            }
            Action::Cancel => self.mode = Mode::Normal,
            Action::Submit => self.submit(),
            Action::Pull => {
                self.report("pull", git::pull());
                self.refresh();
            }
            Action::Push => {
                self.report("push", git::push());
                self.refresh();
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let last = self.rows.len() as i32 - 1;
        self.selected = (self.selected as i32 + delta).clamp(0, last) as usize;
        self.reload_diff();
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sidebar => Focus::Diff,
            Focus::Diff => Focus::Sidebar,
        };
    }

    fn enter_row(&mut self) {
        match self.rows.get(self.selected).map(|r| (r.id, r.kind)) {
            Some((id, RowKind::Dir)) => self.set_collapsed(id, !self.collapsed.contains(&id)),
            Some((_, RowKind::File { .. })) => self.focus = Focus::Diff,
            None => {}
        }
    }

    fn collapse(&mut self) {
        let Some(row) = self.rows.get(self.selected).cloned() else {
            return;
        };
        if matches!(row.kind, RowKind::Dir) && row.expanded {
            self.set_collapsed(row.id, true);
        } else if row.depth > 0
            && let Some(parent) = (0..self.selected)
                .rev()
                .find(|&i| self.rows[i].depth < row.depth)
        {
            self.selected = parent;
            self.reload_diff();
        }
    }

    fn expand(&mut self) {
        let Some(row) = self.rows.get(self.selected).cloned() else {
            return;
        };
        if !matches!(row.kind, RowKind::Dir) {
            return;
        }
        if row.expanded {
            if self
                .rows
                .get(self.selected + 1)
                .is_some_and(|c| c.depth > row.depth)
            {
                self.selected += 1;
                self.reload_diff();
            }
        } else {
            self.set_collapsed(row.id, false);
        }
    }

    fn set_collapsed(&mut self, id: usize, collapsed: bool) {
        if collapsed {
            self.collapsed.insert(id);
        } else {
            self.collapsed.remove(&id);
        }
        self.reflatten();
    }

    fn reflatten(&mut self) {
        let keep = self.rows.get(self.selected).map(|r| r.id);
        self.rows = tree::flatten(&self.tree, &self.collapsed);
        self.selected = keep
            .and_then(|id| self.rows.iter().position(|r| r.id == id))
            .unwrap_or_else(|| self.selected.min(self.rows.len().saturating_sub(1)));
        self.reload_diff();
    }

    fn input_mut(&mut self) -> Option<&mut Input> {
        match &mut self.mode {
            Mode::Commit(input) | Mode::Branch(input) => Some(input),
            _ => None,
        }
    }

    fn submit(&mut self) {
        match std::mem::replace(&mut self.mode, Mode::Normal) {
            Mode::Commit(input) => self.report_unit("commit", git::commit(&input.value)),
            Mode::Branch(input) => self.report_unit("branch", git::checkout(&input.value)),
            Mode::Confirm => self.report_unit("reset", git::reset_hard_clean()),
            Mode::Normal | Mode::Help => {}
        }
        self.refresh();
    }

    fn stage_toggle(&mut self) {
        let Some(row) = self.rows.get(self.selected).cloned() else {
            return;
        };
        let result = match row.kind {
            RowKind::File { index } => {
                let file = &self.status.files[index];
                if file.staged {
                    git::unstage(&file.path)
                } else {
                    git::stage(&file.path)
                }
            }
            RowKind::Dir => {
                if self.files_under(&row.path).any(|f| !f.staged) {
                    git::stage(&row.path)
                } else {
                    git::unstage(&row.path)
                }
            }
        };
        if let Err(e) = result {
            self.message = e.to_string();
        }
        self.refresh();
    }

    fn files_under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = &'a FileEntry> {
        let dir = format!("{prefix}/");
        self.status
            .files
            .iter()
            .filter(move |f| f.path == prefix || f.path.starts_with(&dir))
    }

    fn refresh(&mut self) {
        match git::status_raw() {
            Ok(bytes) => {
                self.status = status::parse(&bytes);
                self.tree = tree::build(&self.status.files);
                self.rows = tree::flatten(&self.tree, &self.collapsed);
                if self.selected >= self.rows.len() {
                    self.selected = self.rows.len().saturating_sub(1);
                }
                self.reload_diff();
            }
            Err(e) => self.message = e.to_string(),
        }
    }

    fn reload_diff(&mut self) {
        self.diff_scroll = 0;
        self.diff_rows = match self
            .rows
            .get(self.selected)
            .map(|r| (r.kind, r.path.clone()))
        {
            Some((RowKind::File { index }, _)) => {
                let raw = self
                    .status
                    .files
                    .get(index)
                    .map(entry_diff)
                    .unwrap_or_default();
                diff::parse(&raw, false)
            }
            Some((RowKind::Dir, path)) => diff::parse(&self.folder_diff(&path), true),
            None => Vec::new(),
        };
    }

    fn folder_diff(&self, prefix: &str) -> String {
        let mut out = String::new();
        for file in self.files_under(prefix) {
            let chunk = entry_diff(file);
            out.push_str(&chunk);
            if !chunk.is_empty() && !chunk.ends_with('\n') {
                out.push('\n');
            }
        }
        out
    }

    fn report(&mut self, label: &str, result: Result<String>) {
        self.message = match result {
            Ok(text) => format!("{label}: {text}"),
            Err(e) => format!("{label}: {e}"),
        };
    }

    fn report_unit(&mut self, label: &str, result: Result<()>) {
        self.message = match result {
            Ok(()) => format!("{label}: ok"),
            Err(e) => format!("{label}: {e}"),
        };
    }
}

fn entry_diff(file: &FileEntry) -> String {
    if file.untracked {
        git::diff_untracked(&file.path)
    } else if file.unstaged {
        git::diff(&file.path, false)
    } else {
        git::diff(&file.path, true)
    }
}

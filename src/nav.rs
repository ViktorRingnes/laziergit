use crate::app::{App, Focus};
use crate::layout;
use ratatui::layout::Rect;

const SCROLL_STEP: i32 = 3;

impl App {
    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
        let max = self.max_diff_scroll();
        if self.diff_scroll > max {
            self.diff_scroll = max;
        }
        self.ensure_visible();
    }

    pub fn push_digit(&mut self, digit: u32) {
        if digit == 0 && self.count.is_none() {
            return;
        }
        let next = self.count.unwrap_or(0).saturating_mul(10).saturating_add(digit);
        self.count = Some(next.min(99_999));
    }

    pub fn take_count(&mut self) -> i32 {
        self.count.take().unwrap_or(1).max(1) as i32
    }

    pub fn clear_pending(&mut self) {
        self.count = None;
        self.awaiting_g = false;
    }

    pub fn pending_label(&self) -> Option<String> {
        if let Some(count) = self.count {
            Some(count.to_string())
        } else if self.awaiting_g {
            Some("g".to_string())
        } else {
            None
        }
    }

    pub fn press_g(&mut self) {
        if self.awaiting_g {
            self.awaiting_g = false;
            let row = self.count.take();
            self.go_to_line(row, false);
        } else {
            self.awaiting_g = true;
        }
    }

    pub fn go_bottom(&mut self) {
        let row = self.count.take();
        self.go_to_line(row, true);
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.rows.is_empty() {
            return;
        }
        let last = self.rows.len() as i32 - 1;
        let target = (self.selected as i32 + delta).clamp(0, last) as usize;
        self.set_selected(target);
    }

    pub fn set_selected(&mut self, index: usize) {
        if self.rows.is_empty() {
            return;
        }
        let clamped = index.min(self.rows.len() - 1);
        if clamped != self.selected {
            self.selected = clamped;
            self.reload_diff();
        }
        self.ensure_visible();
    }

    pub fn scroll_diff(&mut self, lines: i32) {
        let max = self.max_diff_scroll() as i32;
        self.diff_scroll = (self.diff_scroll as i32 + lines).clamp(0, max) as u16;
    }

    pub fn wheel_step(&self) -> i32 {
        SCROLL_STEP
    }

    pub fn ensure_visible(&mut self) {
        let height = layout::regions(self.area).sidebar_inner.height as usize;
        if height == 0 || self.rows.is_empty() {
            return;
        }
        if self.selected < self.sidebar_offset {
            self.sidebar_offset = self.selected;
        } else if self.selected >= self.sidebar_offset + height {
            self.sidebar_offset = self.selected + 1 - height;
        }
        let max_offset = self.rows.len().saturating_sub(height);
        self.sidebar_offset = self.sidebar_offset.min(max_offset);
    }

    pub fn max_diff_scroll(&self) -> u16 {
        let height = layout::regions(self.area).diff_inner.height as usize;
        self.diff_rows.len().saturating_sub(height) as u16
    }

    fn go_to_line(&mut self, row: Option<u32>, default_bottom: bool) {
        match self.focus {
            Focus::Sidebar => {
                let last = self.rows.len().saturating_sub(1);
                let target = match row {
                    Some(r) => (r as usize).saturating_sub(1).min(last),
                    None if default_bottom => last,
                    None => 0,
                };
                self.set_selected(target);
            }
            Focus::Diff => {
                let max = self.max_diff_scroll();
                self.diff_scroll = match row {
                    Some(r) => (r.saturating_sub(1) as u16).min(max),
                    None if default_bottom => max,
                    None => 0,
                };
            }
        }
    }
}

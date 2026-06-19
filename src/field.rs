#[derive(Clone, Copy)]
pub enum EditOp {
    Insert(char),
    Backspace,
    Left,
    Right,
}

#[derive(Default)]
pub struct Input {
    pub value: String,
    pub cursor: usize,
}

impl Input {
    pub fn apply(&mut self, op: EditOp) {
        match op {
            EditOp::Insert(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
            }
            EditOp::Backspace => {
                if let Some(prev) = self.value[..self.cursor].chars().next_back() {
                    self.cursor -= prev.len_utf8();
                    self.value.remove(self.cursor);
                }
            }
            EditOp::Left => {
                if let Some(prev) = self.value[..self.cursor].chars().next_back() {
                    self.cursor -= prev.len_utf8();
                }
            }
            EditOp::Right => {
                if let Some(next) = self.value[self.cursor..].chars().next() {
                    self.cursor += next.len_utf8();
                }
            }
        }
    }

    pub fn cursor_col(&self) -> u16 {
        self.value[..self.cursor].chars().count() as u16
    }
}

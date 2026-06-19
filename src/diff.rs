const TAB: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Context,
    Del,
    Add,
    Empty,
    Hunk,
    File,
}

#[derive(Clone)]
pub struct Cell {
    pub kind: Kind,
    pub num: Option<u32>,
    pub text: String,
}

impl Cell {
    fn line(kind: Kind, num: u32, text: &str) -> Cell {
        Cell {
            kind,
            num: Some(num),
            text: text.to_string(),
        }
    }

    fn empty() -> Cell {
        Cell {
            kind: Kind::Empty,
            num: None,
            text: String::new(),
        }
    }
}

#[derive(Clone)]
pub struct DiffRow {
    pub left: Cell,
    pub right: Cell,
}

impl DiffRow {
    fn banner(kind: Kind, text: &str) -> DiffRow {
        let cell = Cell {
            kind,
            num: None,
            text: text.to_string(),
        };
        DiffRow {
            left: cell.clone(),
            right: cell,
        }
    }
}

#[derive(Default)]
struct State {
    rows: Vec<DiffRow>,
    old_no: u32,
    new_no: u32,
    del: Vec<String>,
    add: Vec<String>,
}

pub fn parse(diff: &str, file_headers: bool) -> Vec<DiffRow> {
    let mut s = State::default();
    let mut seen_hunk = false;

    for raw in diff.lines() {
        if let Some(path) = file_header(raw) {
            flush(&mut s);
            seen_hunk = false;
            if file_headers {
                s.rows.push(DiffRow::banner(Kind::File, path));
            }
            continue;
        }
        if let Some((old_start, new_start)) = parse_hunk_header(raw) {
            flush(&mut s);
            seen_hunk = true;
            s.old_no = old_start;
            s.new_no = new_start;
            s.rows.push(DiffRow::banner(Kind::Hunk, raw));
            continue;
        }
        if !seen_hunk {
            if raw.starts_with("Binary ") || raw.starts_with("GIT binary") {
                s.rows
                    .push(DiffRow::banner(Kind::Hunk, "Binary file differs"));
            }
            continue;
        }
        match raw.as_bytes().first() {
            Some(b'-') => s.del.push(prep(&raw[1..])),
            Some(b'+') => s.add.push(prep(&raw[1..])),
            Some(b'\\') => {}
            Some(b' ') => context(&mut s, &prep(&raw[1..])),
            _ => context(&mut s, ""),
        }
    }
    flush(&mut s);
    s.rows
}

fn context(s: &mut State, text: &str) {
    flush(s);
    let left = Cell::line(Kind::Context, s.old_no, text);
    let right = Cell::line(Kind::Context, s.new_no, text);
    s.old_no += 1;
    s.new_no += 1;
    s.rows.push(DiffRow { left, right });
}

fn flush(s: &mut State) {
    if s.del.len() == s.add.len() {
        for i in 0..s.del.len() {
            let left = Cell::line(Kind::Del, s.old_no, &s.del[i]);
            let right = Cell::line(Kind::Add, s.new_no, &s.add[i]);
            s.old_no += 1;
            s.new_no += 1;
            s.rows.push(DiffRow { left, right });
        }
    } else {
        for text in &s.del {
            let left = Cell::line(Kind::Del, s.old_no, text);
            s.old_no += 1;
            s.rows.push(DiffRow {
                left,
                right: Cell::empty(),
            });
        }
        for text in &s.add {
            let right = Cell::line(Kind::Add, s.new_no, text);
            s.new_no += 1;
            s.rows.push(DiffRow {
                left: Cell::empty(),
                right,
            });
        }
    }
    s.del.clear();
    s.add.clear();
}

fn prep(s: &str) -> String {
    let s = s.strip_suffix('\r').unwrap_or(s);
    let mut out = String::with_capacity(s.len());
    let mut col = 0;
    for c in s.chars() {
        if c == '\t' {
            let n = TAB - (col % TAB);
            out.extend(std::iter::repeat_n(' ', n));
            col += n;
        } else {
            out.push(c);
            col += 1;
        }
    }
    out
}

fn file_header(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("diff --git ")?;
    let idx = rest.find(" b/")?;
    Some(&rest[idx + 3..])
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    let rest = line.strip_prefix("@@ -")?;
    let (ranges, _) = rest.split_once(" @@")?;
    let (old, new) = ranges.split_once(" +")?;
    let old_start = old.split(',').next()?.parse().ok()?;
    let new_start = new.split(',').next()?.parse().ok()?;
    Some((old_start, new_start))
}

pub fn gutter_width(rows: &[DiffRow]) -> usize {
    let max = rows
        .iter()
        .flat_map(|r| [r.left.num, r.right.num])
        .flatten()
        .max()
        .unwrap_or(0);
    max.to_string().len().clamp(2, 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_equal_runs_and_stacks_unequal() {
        let diff = "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1,3 +1,4 @@\n a\n-b\n+B\n+C\n c\n";
        let rows = parse(diff, false);

        // hunk, context a, [del b | empty], [empty | add B], [empty | add C], context c
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[0].left.kind, Kind::Hunk);

        assert_eq!(rows[1].left.kind, Kind::Context);
        assert_eq!(rows[1].left.num, Some(1));
        assert_eq!(rows[1].left.text, "a");

        assert_eq!(rows[2].left.kind, Kind::Del);
        assert_eq!(rows[2].left.text, "b");
        assert_eq!(rows[2].right.kind, Kind::Empty);

        assert_eq!(rows[3].right.kind, Kind::Add);
        assert_eq!(rows[3].right.text, "B");
        assert_eq!(rows[4].right.text, "C");

        assert_eq!(rows[5].left.num, Some(3));
        assert_eq!(rows[5].right.num, Some(4));
    }

    #[test]
    fn zips_balanced_replacement_side_by_side() {
        let diff = "@@ -5,1 +5,1 @@\n-old line\n+new line\n";
        let rows = parse(diff, false);

        assert_eq!(rows.len(), 2);
        let change = &rows[1];
        assert_eq!(change.left.kind, Kind::Del);
        assert_eq!(change.left.num, Some(5));
        assert_eq!(change.left.text, "old line");
        assert_eq!(change.right.kind, Kind::Add);
        assert_eq!(change.right.num, Some(5));
        assert_eq!(change.right.text, "new line");
    }

    #[test]
    fn multi_file_emits_banners_only_when_enabled() {
        let diff = "diff --git a/x.rs b/x.rs\n--- a/x.rs\n+++ b/x.rs\n@@ -1 +1 @@\n-a\n+b\ndiff --git a/y.rs b/y.rs\n--- a/y.rs\n+++ b/y.rs\n@@ -1 +1 @@\n-c\n+d\n";

        let with = parse(diff, true);
        let banners: Vec<&str> = with
            .iter()
            .filter(|r| r.left.kind == Kind::File)
            .map(|r| r.left.text.as_str())
            .collect();
        assert_eq!(banners, vec!["x.rs", "y.rs"]);

        let without = parse(diff, false);
        assert!(without.iter().all(|r| r.left.kind != Kind::File));
    }
}

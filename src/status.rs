#[derive(Default)]
pub struct Branch {
    pub name: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

impl Kind {
    fn from_char(c: char) -> Self {
        match c {
            'A' => Kind::Added,
            'D' => Kind::Deleted,
            'R' | 'C' => Kind::Renamed,
            'U' => Kind::Conflicted,
            '?' => Kind::Untracked,
            _ => Kind::Modified,
        }
    }

    pub fn letter(self) -> char {
        match self {
            Kind::Modified => 'M',
            Kind::Added => 'A',
            Kind::Deleted => 'D',
            Kind::Renamed => 'R',
            Kind::Untracked => '?',
            Kind::Conflicted => 'U',
        }
    }
}

pub struct FileEntry {
    pub path: String,
    pub staged: bool,
    pub unstaged: bool,
    pub untracked: bool,
    pub kind: Kind,
}

#[derive(Default)]
pub struct RepoStatus {
    pub branch: Branch,
    pub files: Vec<FileEntry>,
}

pub fn parse(bytes: &[u8]) -> RepoStatus {
    let text = String::from_utf8_lossy(bytes);
    let tokens: Vec<&str> = text.split('\0').filter(|t| !t.is_empty()).collect();

    let mut status = RepoStatus::default();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];
        if let Some(rest) = token.strip_prefix("# ") {
            apply_branch(&mut status.branch, rest);
        } else if let Some(rest) = token.strip_prefix("1 ") {
            status.files.push(changed(rest, 6));
        } else if let Some(rest) = token.strip_prefix("2 ") {
            status.files.push(changed(rest, 7));
            i += 1; // rename records carry the original path in a following NUL field
        } else if let Some(path) = token.strip_prefix("? ") {
            status.files.push(untracked(path));
        } else if let Some(rest) = token.strip_prefix("u ") {
            status.files.push(unmerged(rest));
        }
        i += 1;
    }
    status
}

fn changed(rest: &str, meta_fields: usize) -> FileEntry {
    let mut parts = rest.splitn(meta_fields + 2, ' ');
    let xy = parts.next().unwrap_or("..");
    let path = parts.nth(meta_fields).unwrap_or("");
    from_xy(xy, path)
}

fn from_xy(xy: &str, path: &str) -> FileEntry {
    let mut chars = xy.chars();
    let x = chars.next().unwrap_or('.');
    let y = chars.next().unwrap_or('.');
    let staged = x != '.';
    let unstaged = y != '.';
    FileEntry {
        path: path.to_string(),
        staged,
        unstaged,
        untracked: false,
        kind: Kind::from_char(if unstaged { y } else { x }),
    }
}

fn untracked(path: &str) -> FileEntry {
    FileEntry {
        path: path.to_string(),
        staged: false,
        unstaged: true,
        untracked: true,
        kind: Kind::Untracked,
    }
}

fn unmerged(rest: &str) -> FileEntry {
    FileEntry {
        staged: false,
        unstaged: true,
        kind: Kind::Conflicted,
        ..changed(rest, 8)
    }
}

fn apply_branch(branch: &mut Branch, rest: &str) {
    if let Some(v) = rest.strip_prefix("branch.head ") {
        branch.name = v.to_string();
    } else if let Some(v) = rest.strip_prefix("branch.upstream ") {
        branch.upstream = Some(v.to_string());
    } else if let Some(v) = rest.strip_prefix("branch.ab ") {
        for token in v.split(' ') {
            if let Some(n) = token.strip_prefix('+') {
                branch.ahead = n.parse().unwrap_or(0);
            } else if let Some(n) = token.strip_prefix('-') {
                branch.behind = n.parse().unwrap_or(0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_ordinary_and_untracked() {
        let raw = b"# branch.head main\x00# branch.ab +1 -2\x001 .M N... 100644 100644 100644 aaaa bbbb src/main.rs\x00? new file.txt\x00";
        let status = parse(raw);

        assert_eq!(status.branch.name, "main");
        assert_eq!(status.branch.ahead, 1);
        assert_eq!(status.branch.behind, 2);
        assert_eq!(status.files.len(), 2);

        let modified = &status.files[0];
        assert_eq!(modified.path, "src/main.rs");
        assert!(modified.unstaged && !modified.staged);
        assert_eq!(modified.kind, Kind::Modified);

        let untracked = &status.files[1];
        assert_eq!(untracked.path, "new file.txt");
        assert!(untracked.untracked);
    }

    #[test]
    fn parses_staged_and_rename_with_spaces() {
        let raw = b"1 A. N... 000000 100644 100644 0000 cccc added.rs\x002 R. N... 100644 100644 100644 dddd eeee R100 new name.rs\x00old name.rs\x00";
        let status = parse(raw);

        assert_eq!(status.files.len(), 2);

        let added = &status.files[0];
        assert_eq!(added.path, "added.rs");
        assert!(added.staged && !added.unstaged);
        assert_eq!(added.kind, Kind::Added);

        let renamed = &status.files[1];
        assert_eq!(renamed.path, "new name.rs");
        assert_eq!(renamed.kind, Kind::Renamed);
    }

    #[test]
    fn parses_unmerged_conflict_with_spaces() {
        let raw = b"u UU N... 100644 100644 100644 100644 aaaa bbbb cccc conflicted file.txt\x00";
        let status = parse(raw);

        assert_eq!(status.files.len(), 1);
        assert_eq!(status.files[0].path, "conflicted file.txt");
        assert_eq!(status.files[0].kind, Kind::Conflicted);
    }
}

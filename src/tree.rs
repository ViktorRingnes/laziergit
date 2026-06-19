use crate::status::FileEntry;
use std::collections::HashSet;

pub struct TreeNode {
    pub id: usize,
    pub name: String,
    pub path: String,
    pub file: Option<usize>,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn is_dir(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(Clone, Copy)]
pub enum RowKind {
    Dir,
    File { index: usize },
}

#[derive(Clone)]
pub struct Row {
    pub id: usize,
    pub depth: u16,
    pub is_last: bool,
    pub expanded: bool,
    pub kind: RowKind,
    pub path: String,
    pub name: String,
}

struct Builder {
    name: String,
    path: String,
    file: Option<usize>,
    children: Vec<Builder>,
}

pub fn build(files: &[FileEntry]) -> Vec<TreeNode> {
    let mut roots: Vec<Builder> = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let segments: Vec<&str> = file.path.split('/').collect();
        insert(&mut roots, &segments, "", index);
    }
    sort(&mut roots);
    let mut next_id = 0;
    roots
        .into_iter()
        .map(|b| finalize(b, &mut next_id))
        .collect()
}

fn insert(level: &mut Vec<Builder>, segments: &[&str], prefix: &str, index: usize) {
    let name = segments[0];
    let path = if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    };
    let pos = level
        .iter()
        .position(|b| b.name == name)
        .unwrap_or_else(|| {
            level.push(Builder {
                name: name.to_string(),
                path: path.clone(),
                file: None,
                children: Vec::new(),
            });
            level.len() - 1
        });
    if segments.len() == 1 {
        level[pos].file = Some(index);
    } else {
        insert(&mut level[pos].children, &segments[1..], &path, index);
    }
}

fn sort(level: &mut [Builder]) {
    level.sort_by(|a, b| {
        let a_dir = !a.children.is_empty();
        let b_dir = !b.children.is_empty();
        b_dir.cmp(&a_dir).then_with(|| a.name.cmp(&b.name))
    });
    for node in level.iter_mut() {
        sort(&mut node.children);
    }
}

fn finalize(builder: Builder, next_id: &mut usize) -> TreeNode {
    let id = *next_id;
    *next_id += 1;
    let children = builder
        .children
        .into_iter()
        .map(|c| finalize(c, next_id))
        .collect();
    TreeNode {
        id,
        name: builder.name,
        path: builder.path,
        file: builder.file,
        children,
    }
}

pub fn flatten(forest: &[TreeNode], collapsed: &HashSet<usize>) -> Vec<Row> {
    let mut rows = Vec::new();
    walk(forest, 0, collapsed, &mut rows);
    rows
}

fn walk(nodes: &[TreeNode], depth: u16, collapsed: &HashSet<usize>, out: &mut Vec<Row>) {
    let last = nodes.len().saturating_sub(1);
    for (i, node) in nodes.iter().enumerate() {
        let is_dir = node.is_dir();
        let expanded = is_dir && !collapsed.contains(&node.id);
        let kind = match node.file {
            Some(index) if !is_dir => RowKind::File { index },
            _ => RowKind::Dir,
        };
        out.push(Row {
            id: node.id,
            depth,
            is_last: i == last,
            expanded,
            kind,
            path: node.path.clone(),
            name: node.name.clone(),
        });
        if expanded {
            walk(&node.children, depth + 1, collapsed, out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::Kind;

    fn entry(path: &str) -> FileEntry {
        FileEntry {
            path: path.to_string(),
            staged: false,
            unstaged: true,
            untracked: false,
            kind: Kind::Modified,
        }
    }

    #[test]
    fn builds_tree_dirs_first_and_flattens_in_order() {
        let files = vec![entry("src/a.rs"), entry("src/b.rs"), entry("README.md")];
        let forest = build(&files);
        let rows = flatten(&forest, &HashSet::new());

        // dirs first: src/ folder (a.rs, b.rs) then README.md
        assert_eq!(rows.len(), 4);
        assert!(matches!(rows[0].kind, RowKind::Dir));
        assert_eq!(rows[0].name, "src");
        assert_eq!(rows[1].depth, 1);
        assert!(matches!(rows[1].kind, RowKind::File { .. }));
        assert_eq!(rows[3].name, "README.md");
        assert_eq!(rows[3].depth, 0);
        assert!(rows[3].is_last);
    }

    #[test]
    fn collapsing_a_dir_hides_children() {
        let files = vec![entry("src/a.rs"), entry("src/b.rs"), entry("README.md")];
        let forest = build(&files);
        let dir_id = forest[0].id;
        let rows = flatten(&forest, &HashSet::from([dir_id]));

        assert_eq!(rows.len(), 2);
        assert!(matches!(rows[0].kind, RowKind::Dir));
        assert!(!rows[0].expanded);
        assert_eq!(rows[1].name, "README.md");
    }
}

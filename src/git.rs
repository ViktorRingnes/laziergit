use color_eyre::eyre::{Result, bail};
use std::process::Command;

fn run(args: &[&str]) -> Result<Vec<u8>> {
    let out = Command::new("git").args(args).output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(out.stdout)
}

fn capture(args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

pub fn status_raw() -> Result<Vec<u8>> {
    run(&["status", "--porcelain=v2", "--branch", "-z", "-uall"])
}

pub fn diff(path: &str, staged: bool) -> String {
    if staged {
        capture(&["diff", "--no-color", "--cached", "--", path])
    } else {
        capture(&["diff", "--no-color", "--", path])
    }
}

pub fn diff_untracked(path: &str) -> String {
    capture(&["diff", "--no-color", "--no-index", "--", "/dev/null", path])
}

pub fn stage(pathspec: &str) -> Result<()> {
    run(&["add", "--", pathspec]).map(drop)
}

pub fn unstage(pathspec: &str) -> Result<()> {
    if run(&["restore", "--staged", "--", pathspec]).is_ok() {
        return Ok(());
    }
    run(&["rm", "--cached", "-r", "-q", "--", pathspec]).map(drop)
}

pub fn commit(message: &str) -> Result<()> {
    run(&["commit", "-m", message]).map(drop)
}

pub fn checkout(name: &str) -> Result<()> {
    if run(&["checkout", name]).is_ok() {
        return Ok(());
    }
    run(&["checkout", "-b", name]).map(drop)
}

pub fn reset_hard_clean() -> Result<()> {
    run(&["reset", "--hard"])?;
    run(&["clean", "-fd"])?;
    Ok(())
}

pub fn pull() -> Result<String> {
    remote(&["pull"])
}

pub fn push() -> Result<String> {
    remote(&["push"])
}

fn remote(args: &[&str]) -> Result<String> {
    let out = Command::new("git").args(args).output()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let last = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("done")
        .trim()
        .to_string();
    if out.status.success() {
        Ok(last)
    } else {
        bail!("{last}");
    }
}

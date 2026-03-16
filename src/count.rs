use std::{ffi::OsStr, path::Path};

use crate::{BOLD, RED, RESET, Result};

const LINT_ATTR_PREFIXES: [&str; 8] = [
    "allow(",
    "warn(",
    "deny(",
    "forbid(",
    "expect(",
    "#[cfg_attr(any(windows, target_os = \"wasi\"), expect(",
    "cfg_attr(test,",
    "cfg_attr(fuzzing,",
];

struct Counts {
    lines: usize,
    production: usize,
    lint: usize,
    tests: usize,
    comments: usize,
    blanks: usize,
}

fn is_lint_attr(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("#![")
        .or_else(|| trimmed.strip_prefix("#["))
        .is_some_and(|after| {
            LINT_ATTR_PREFIXES
                .iter()
                .any(|prefix| after.starts_with(prefix))
        })
}

fn count_file(file: &Path) -> Result<Counts> {
    if file.extension() != Some(OsStr::new("rs")) {
        Err(format!("{} is not a Rust file", file.display()))?;
    }

    let source = std::fs::read_to_string(file)?;
    let mut blanks = 0;
    let mut comments = 0;
    let mut lint = 0;
    let mut production = 0;
    let mut tests = 0;
    let mut in_tests = false;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blanks += 1;
        } else if trimmed.starts_with("//") {
            comments += 1;
        } else if !in_tests && trimmed.starts_with("#[cfg(test)]") {
            in_tests = true;
            tests += 1;
        } else if is_lint_attr(line) {
            lint += 1;
        } else if in_tests {
            tests += 1;
        } else {
            production += 1;
        }
    }

    let lines = production + lint + tests + comments + blanks;

    Ok(Counts { lines, production, lint, tests, comments, blanks })
}

pub fn count_rust_lines(root: &Path) -> Result<()> {
    if !root.is_dir() {
        Err(format!("{}: not a directory", root.display()))?;
    }

    let mut file_counts: Vec<(String, Counts)> = Vec::new();
    let mut pending_dirs = vec![root.to_path_buf()];
    while let Some(dir) = pending_dirs.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                let name = entry.file_name();
                if name != "target" && name != ".git" {
                    pending_dirs.push(path);
                }
                continue;
            }
            if path.extension() != Some(OsStr::new("rs")) {
                continue;
            }

            let counts = count_file(&path)?;
            let relative_path = path.strip_prefix(root)?.display().to_string();
            file_counts.push((relative_path, counts));
        }
    }
    file_counts.sort_by(|a, b| a.0.cmp(&b.0));

    print_report(&file_counts)
}

fn print_report(file_counts: &[(String, Counts)]) -> Result<()> {
    let w = file_counts
        .iter()
        .map(|(path, _)| path.len())
        .max()
        .unwrap_or_default()
        .max(5);

    let sum = |f: fn(&Counts) -> usize| -> usize {
        file_counts.iter().map(|(_, c)| f(c)).sum()
    };

    const G: &str = "\x1b[32m";
    const P: &str = "\x1b[35m";

    println!();
    println!(
        "  {G}{BOLD}{:w$}{RESET}  {P}{BOLD}{:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}{RESET}",
        "File", "Lines", "Prod", "Lints", "Tests", "Comments", "Blanks",
    );
    println!("  {G}{}{RESET}", "─".repeat(w + 60));

    for (path, c) in file_counts {
        println!(
            "  {G}{:w$}{RESET}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}",
            path, c.lines, c.production, c.lint, c.tests, c.comments, c.blanks,
        );
    }

    println!("  {G}{}{RESET}", "─".repeat(w + 60));
    println!(
        "  {G}{BOLD}{:w$}{RESET}  {BOLD}{:>8}  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}{RESET}",
        "Total",
        sum(|c| c.lines),
        sum(|c| c.production),
        sum(|c| c.lint),
        sum(|c| c.tests),
        sum(|c| c.comments),
        sum(|c| c.blanks),
    );
    println!();

    if sum(|c| c.lines) == 0 {
        eprintln!("{RED}warning{RESET}: no Rust source files found");
    }

    Ok(())
}

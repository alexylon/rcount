# rloc (Rust Lines of Code)

Count Rust lines of code in a project. Recursively finds all `.rs` files, skipping `target/` and `.git/`.

Reports a per-file breakdown with the following columns:

| Column | Description |
|---|---|
| **Lines** | Total lines in the file |
| **Prod** | Production code (excludes tests, lint attributes, comments, and blanks) |
| **Lints** | Lint attribute lines (`allow`, `warn`, `deny`, `forbid`, `expect`, `cfg_attr`) |
| **Tests** | Code lines after `#[cfg(test)]` |
| **Comments** | Comment lines (`//`) |
| **Blanks** | Empty lines |

## Usage

```bash
# Count lines in current directory
rloc

# Count lines in another project
rloc path/to/project
```

## Example output

```
  File            Lines      Prod     Lints     Tests  Comments    Blanks
  ────────────────────────────────────────────────────────────────────────
  src/count.rs      155       131         0         0         3        21
  src/main.rs        22        17         0         0         0         5
  ────────────────────────────────────────────────────────────────────────
  Total             177       148         0         0         3        26
```

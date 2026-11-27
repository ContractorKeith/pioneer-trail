use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{bail, Context, Result};
use clap::Parser;

const BASELINE_FILE: &str = "comment-baseline.txt";
const UPDATE_CMD: &str = "cargo run -p pioneer-tools --bin comment-ratchet -- --update";

#[derive(Parser)]
#[command(about = "Bans comments in tracked .rs and .py files, with a per-file ratchet baseline")]
struct Cli {
    #[arg(long, conflicts_with = "list")]
    update: bool,
    #[arg(long)]
    list: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Comment {
    line: usize,
    text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Violation {
    Increased { path: String, baseline: usize, current: usize },
    Reduced { path: String, baseline: usize, current: usize },
    Gone { path: String },
}

impl Violation {
    fn path(&self) -> &str {
        match self {
            Self::Increased { path, .. } | Self::Reduced { path, .. } | Self::Gone { path } => path,
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn skip_quoted(chars: &[char], start: usize, line: &mut usize) -> usize {
    let mut i = start + 1;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                if chars.get(i + 1) == Some(&'\n') {
                    *line += 1;
                }
                i += 2;
            }
            '"' => return i + 1,
            '\n' => {
                *line += 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    chars.len()
}

fn raw_string_end(chars: &[char], start: usize, line: &mut usize) -> Option<usize> {
    let hashes = chars[start..].iter().take_while(|&&c| c == '#').count();
    if chars.get(start + hashes) != Some(&'"') {
        return None;
    }
    let mut i = start + hashes + 1;
    while i < chars.len() {
        if chars[i] == '\n' {
            *line += 1;
        } else if chars[i] == '"' && chars[i + 1..].iter().take(hashes).all(|&c| c == '#') {
            let closing = chars[i + 1..].iter().take_while(|&&c| c == '#').count();
            if closing >= hashes {
                return Some(i + 1 + hashes);
            }
        }
        i += 1;
    }
    Some(chars.len())
}

fn skip_char_or_lifetime(chars: &[char], start: usize, line: &mut usize) -> usize {
    match (chars.get(start + 1), chars.get(start + 2)) {
        (Some('\\'), _) => {
            let mut j = start + 3;
            while j < chars.len() && chars[j] != '\'' && chars[j] != '\n' {
                j += 1;
            }
            if chars.get(j) == Some(&'\'') {
                j + 1
            } else {
                j
            }
        }
        (Some(&c), Some('\'')) => {
            if c == '\n' {
                *line += 1;
            }
            start + 3
        }
        _ => start + 1,
    }
}

fn scan_rust(src: &str) -> Vec<Comment> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut line = 1;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            '/' if next == Some('/') => {
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                out.push(Comment { line, text: chars[start..i].iter().collect() });
            }
            '/' if next == Some('*') => {
                let (start, start_line) = (i, line);
                let mut depth = 0usize;
                while i < chars.len() {
                    let pair = (chars[i], chars.get(i + 1).copied());
                    if pair == ('/', Some('*')) {
                        depth += 1;
                        i += 2;
                    } else if pair == ('*', Some('/')) {
                        depth -= 1;
                        i += 2;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        if chars[i] == '\n' {
                            line += 1;
                        }
                        i += 1;
                    }
                }
                out.push(Comment {
                    line: start_line,
                    text: chars[start..i.min(chars.len())].iter().collect(),
                });
            }
            '"' => i = skip_quoted(&chars, i, &mut line),
            '\'' => i = skip_char_or_lifetime(&chars, i, &mut line),
            c if is_word_char(c) => {
                let start = i;
                while i < chars.len() && is_word_char(chars[i]) {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let after = chars.get(i).copied();
                match word.as_str() {
                    "r" | "br" | "cr" => {
                        if let Some(end) = raw_string_end(&chars, i, &mut line) {
                            i = end;
                        }
                    }
                    "b" | "c" if after == Some('"') => i = skip_quoted(&chars, i, &mut line),
                    "b" if after == Some('\'') => {
                        i = skip_char_or_lifetime(&chars, i, &mut line);
                    }
                    _ => {}
                }
            }
            _ => i += 1,
        }
    }
    out
}

fn scan_python(src: &str) -> Vec<Comment> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut line = 1;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            '#' => {
                let start = i;
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                let shebang = start == 0 && chars.get(1) == Some(&'!');
                if !shebang {
                    out.push(Comment { line, text: chars[start..i].iter().collect() });
                }
            }
            '"' | '\'' => {
                let triple = chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c);
                i += if triple { 3 } else { 1 };
                while i < chars.len() {
                    if chars[i] == '\\' {
                        if chars.get(i + 1) == Some(&'\n') {
                            line += 1;
                        }
                        i += 2;
                    } else if chars[i] == c {
                        if !triple {
                            i += 1;
                            break;
                        }
                        if chars.get(i + 1) == Some(&c) && chars.get(i + 2) == Some(&c) {
                            i += 3;
                            break;
                        }
                        i += 1;
                    } else if chars[i] == '\n' {
                        if !triple {
                            break;
                        }
                        line += 1;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
            }
            _ => i += 1,
        }
    }
    out
}

fn scan_source(path: &str, src: &str) -> Vec<Comment> {
    if path.ends_with(".py") {
        scan_python(src)
    } else {
        scan_rust(src)
    }
}

fn parse_baseline(text: &str) -> Result<BTreeMap<String, usize>> {
    let mut map = BTreeMap::new();
    for (n, line) in text.lines().enumerate() {
        let parsed = line
            .split_once(' ')
            .and_then(|(count, path)| Some((count.parse::<usize>().ok()?, path)));
        let Some((count, path)) = parsed else {
            bail!("{BASELINE_FILE}:{}: expected `<count> <path>`", n + 1);
        };
        map.insert(path.to_string(), count);
    }
    Ok(map)
}

fn format_baseline(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|(path, count)| format!("{count} {path}\n"))
        .collect()
}

fn check(baseline: &BTreeMap<String, usize>, current: &BTreeMap<String, usize>) -> Vec<Violation> {
    let mut found = Vec::new();
    for (path, &count) in current {
        let allowed = baseline.get(path).copied().unwrap_or(0);
        if count > allowed {
            found.push(Violation::Increased {
                path: path.clone(),
                baseline: allowed,
                current: count,
            });
        } else if count < allowed {
            found.push(Violation::Reduced {
                path: path.clone(),
                baseline: allowed,
                current: count,
            });
        }
    }
    for path in baseline.keys().filter(|p| !current.contains_key(*p)) {
        found.push(Violation::Gone { path: path.clone() });
    }
    found.sort_by(|a, b| a.path().cmp(b.path()));
    found
}

fn plan_update(
    baseline: Option<&BTreeMap<String, usize>>,
    current: &BTreeMap<String, usize>,
) -> Result<BTreeMap<String, usize>, Vec<Violation>> {
    if let Some(baseline) = baseline {
        let increases: Vec<Violation> = check(baseline, current)
            .into_iter()
            .filter(|v| matches!(v, Violation::Increased { .. }))
            .collect();
        if !increases.is_empty() {
            return Err(increases);
        }
    }
    Ok(current
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|(path, &count)| (path.clone(), count))
        .collect())
}

fn describe(violation: &Violation) -> String {
    match violation {
        Violation::Increased { path, baseline, current } => {
            format!("{path}: {current} comments, baseline allows {baseline}")
        }
        Violation::Reduced { path, baseline, current } => {
            format!("{path}: {current} comments, baseline still lists {baseline}")
        }
        Violation::Gone { path } => format!("{path}: listed in baseline but no longer exists"),
    }
}

fn report(violations: &[Violation]) {
    let mut banned = false;
    let mut stale = false;
    for violation in violations {
        eprintln!("{}", describe(violation));
        match violation {
            Violation::Increased { .. } => banned = true,
            Violation::Reduced { .. } | Violation::Gone { .. } => stale = true,
        }
    }
    if banned {
        eprintln!("comments are banned: see docs/adr/ and the /delete-comment skill");
    }
    if stale {
        eprintln!("run `{UPDATE_CMD}` to lock in the reduction");
    }
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("running git rev-parse")?;
    if !out.status.success() {
        bail!("git rev-parse --show-toplevel failed; run inside the repository");
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

fn tracked_sources(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files failed");
    }
    let mut files: Vec<String> = String::from_utf8(out.stdout)?
        .split('\0')
        .filter(|p| p.ends_with(".rs") || p.ends_with(".py"))
        .map(String::from)
        .collect();
    files.sort();
    Ok(files)
}

fn scan_tree(root: &Path) -> Result<Vec<(String, Vec<Comment>)>> {
    let mut scanned = Vec::new();
    for path in tracked_sources(root)? {
        match fs::read_to_string(root.join(&path)) {
            Ok(src) => {
                let comments = scan_source(&path, &src);
                scanned.push((path, comments));
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("reading {path}")),
        }
    }
    Ok(scanned)
}

fn totals(counts: &BTreeMap<String, usize>) -> (usize, usize) {
    let comments = counts.values().sum();
    let files = counts.values().filter(|&&count| count > 0).count();
    (comments, files)
}

fn run(cli: &Cli) -> Result<ExitCode> {
    let root = repo_root()?;
    let scanned = scan_tree(&root)?;

    if cli.list {
        let mut stdout = std::io::stdout().lock();
        for (path, comments) in &scanned {
            for comment in comments {
                let first = comment.text.lines().next().unwrap_or_default();
                if writeln!(stdout, "{path}:{}: {first}", comment.line).is_err() {
                    return Ok(ExitCode::SUCCESS);
                }
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    let current: BTreeMap<String, usize> =
        scanned.iter().map(|(path, comments)| (path.clone(), comments.len())).collect();
    let baseline_path = root.join(BASELINE_FILE);
    let baseline = match fs::read_to_string(&baseline_path) {
        Ok(text) => Some(parse_baseline(&text)?),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("reading {BASELINE_FILE}")),
    };

    if cli.update {
        return match plan_update(baseline.as_ref(), &current) {
            Ok(next) => {
                fs::write(&baseline_path, format_baseline(&next))
                    .with_context(|| format!("writing {BASELINE_FILE}"))?;
                let (comments, files) = totals(&next);
                println!("{comments} comments in {files} files (baseline updated)");
                Ok(ExitCode::SUCCESS)
            }
            Err(increases) => {
                report(&increases);
                eprintln!("refusing to update: the baseline can only go down");
                Ok(ExitCode::FAILURE)
            }
        };
    }

    let Some(baseline) = baseline else {
        bail!("{BASELINE_FILE} not found; create it with `{UPDATE_CMD}`");
    };
    let violations = check(&baseline, &current);
    if violations.is_empty() {
        let (comments, files) = totals(&current);
        println!("{comments} comments in {files} files (baseline)");
        Ok(ExitCode::SUCCESS)
    } else {
        report(&violations);
        Ok(ExitCode::FAILURE)
    }
}

fn main() -> ExitCode {
    match run(&Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("comment-ratchet: {e:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_count(src: &str) -> usize {
        scan_rust(src).len()
    }

    fn py_count(src: &str) -> usize {
        scan_python(src).len()
    }

    fn map(entries: &[(&str, usize)]) -> BTreeMap<String, usize> {
        entries.iter().map(|&(p, n)| (p.to_string(), n)).collect()
    }

    #[test]
    fn rust_line_doc_and_block_comments() {
        assert_eq!(rust_count("let x = 1; // hi\n"), 1);
        assert_eq!(rust_count("/// a\n//! b\nfn f() {}\n"), 2);
        assert_eq!(rust_count("/* a */ /** b */ /*! c */"), 3);
        assert_eq!(rust_count("fn f() {}\n"), 0);
    }

    #[test]
    fn rust_nested_block_is_one_comment() {
        assert_eq!(rust_count("/* a /* b */ c */ fn f() {}"), 1);
        assert_eq!(rust_count("/* a\n/* b */\nc */ // d"), 2);
    }

    #[test]
    fn rust_markers_inside_comments_do_not_double_count() {
        assert_eq!(rust_count("// a /* b\nfn f() {}"), 1);
        assert_eq!(rust_count("/* // */"), 1);
    }

    #[test]
    fn rust_reports_line_numbers() {
        let found = scan_rust("fn f() {}\n/* a\nb */\nlet s = \"x\\\ny\";\n// c\n");
        let lines: Vec<usize> = found.iter().map(|c| c.line).collect();
        assert_eq!(lines, vec![2, 6]);
    }

    #[test]
    fn rust_slashes_in_normal_strings() {
        assert_eq!(rust_count(r#"let u = "http://x.com";"#), 0);
        assert_eq!(rust_count(r#"let u = "a \" // b";"#), 0);
        assert_eq!(rust_count(r#"let u = "/* x */";"#), 0);
        assert_eq!(rust_count(r#"let u = "//"; // real"#), 1);
    }

    #[test]
    fn rust_slashes_in_raw_strings() {
        assert_eq!(rust_count(r#"let u = r"http://x";"#), 0);
        assert_eq!(rust_count(r###"let u = r#"a "// b" c"#;"###), 0);
        assert_eq!(rust_count(r####"let u = r##"a "# // b"##;"####), 0);
        assert_eq!(rust_count("let u = r#\"a\n// b\n\"#;\n// c\n"), 1);
    }

    #[test]
    fn rust_raw_identifier_is_not_a_raw_string() {
        assert_eq!(rust_count("let r#type = 1; // c"), 1);
    }

    #[test]
    fn rust_slashes_in_byte_and_c_strings() {
        assert_eq!(rust_count(r#"let u = b"http://x";"#), 0);
        assert_eq!(rust_count(r#"let u = br"//";"#), 0);
        assert_eq!(rust_count(r##"let u = br#"//"#;"##), 0);
        assert_eq!(rust_count(r#"let u = c"//";"#), 0);
    }

    #[test]
    fn rust_char_literals_and_lifetimes() {
        assert_eq!(rust_count("let c = '/'; let d = '\"'; // real"), 1);
        assert_eq!(rust_count("let c = '/'; let d = '/';"), 0);
        assert_eq!(rust_count("let c = b'/'; let d = b'\\'';"), 0);
        assert_eq!(rust_count("let a = '\\''; let b = '\\\\'; // c"), 1);
        assert_eq!(rust_count("let a = '\\u{2F}'; // c"), 1);
        assert_eq!(rust_count("fn f<'a>(x: &'a str) -> &'a str { x } // c"), 1);
        assert_eq!(rust_count("'a: loop { break 'a; }"), 0);
        assert_eq!(rust_count("fn f<'a, 'b>(x: &'a str, y: &'b str) {}"), 0);
    }

    #[test]
    fn rust_scanner_ignores_its_own_source() {
        assert!(scan_rust(include_str!("comment_ratchet.rs")).is_empty());
    }

    #[test]
    fn python_hash_comments() {
        assert_eq!(py_count("x = 1  # hi\n"), 1);
        assert_eq!(py_count("# a\n# b\nx = 1\n"), 2);
        assert_eq!(py_count("x = 1\n"), 0);
    }

    #[test]
    fn python_hash_inside_strings() {
        assert_eq!(py_count("c = \"#000000\"\n"), 0);
        assert_eq!(py_count("c = '#000000'\n"), 0);
        assert_eq!(py_count("c = \"a\" # real\n"), 1);
        assert_eq!(py_count("c = 'it\\'s # not'\n"), 0);
        assert_eq!(py_count("c = \"say \\\"# no\\\"\"\n"), 0);
    }

    #[test]
    fn python_string_prefixes() {
        assert_eq!(py_count("a = r'#x'\n"), 0);
        assert_eq!(py_count("a = b'#x'\n"), 0);
        assert_eq!(py_count("a = f\"{x}#\"\n"), 0);
        assert_eq!(py_count("a = rb\"#x\"\n"), 0);
        assert_eq!(py_count("a = fr'#{x}'  # c\n"), 1);
    }

    #[test]
    fn python_triple_quoted_strings() {
        assert_eq!(py_count("d = \"\"\"a # b\n# c\n\"quoted\" # d\n\"\"\"\n"), 0);
        assert_eq!(py_count("d = '''a # b'''  # real\n"), 1);
        assert_eq!(py_count("d = \"\"\"\"\"\"  # real\n"), 1);
        assert_eq!(py_count("d = \"\"\"a\"\"\"\n# real\n"), 1);
    }

    #[test]
    fn python_shebang_is_ignored_only_on_line_one() {
        assert_eq!(py_count("#!/usr/bin/env python3\nx = 1\n"), 0);
        assert_eq!(py_count("#!/usr/bin/env python3\n# c\n"), 1);
        assert_eq!(py_count("x = 1\n#!/usr/bin/env python3\n"), 1);
    }

    #[test]
    fn python_reports_line_numbers() {
        let found = scan_python("a = \"\"\"x\ny\"\"\"\n# c\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 3);
    }

    #[test]
    fn source_dispatch_uses_extension() {
        assert_eq!(scan_source("a.py", "# c\n").len(), 1);
        assert_eq!(scan_source("a.rs", "# not a comment\n").len(), 0);
        assert_eq!(scan_source("a.rs", "// c\n").len(), 1);
    }

    #[test]
    fn baseline_round_trips_and_omits_zero() {
        let counts = map(&[("b.rs", 2), ("a.py", 1), ("c.rs", 0)]);
        let text = format_baseline(&counts);
        assert_eq!(text, "1 a.py\n2 b.rs\n");
        assert_eq!(parse_baseline(&text).unwrap(), map(&[("a.py", 1), ("b.rs", 2)]));
        assert!(parse_baseline("nonsense\n").is_err());
        assert!(parse_baseline("x a.rs\n").is_err());
    }

    #[test]
    fn check_passes_when_counts_match() {
        let baseline = map(&[("a.rs", 2)]);
        let current = map(&[("a.rs", 2), ("b.rs", 0)]);
        assert!(check(&baseline, &current).is_empty());
    }

    #[test]
    fn check_fails_on_increase_and_unlisted_file() {
        let baseline = map(&[("a.rs", 2)]);
        let current = map(&[("a.rs", 3), ("b.rs", 1)]);
        assert_eq!(
            check(&baseline, &current),
            vec![
                Violation::Increased { path: "a.rs".into(), baseline: 2, current: 3 },
                Violation::Increased { path: "b.rs".into(), baseline: 0, current: 1 },
            ]
        );
    }

    #[test]
    fn check_fails_on_decrease_and_missing_file() {
        let baseline = map(&[("a.rs", 2), ("gone.rs", 4), ("z.rs", 1)]);
        let current = map(&[("a.rs", 1), ("z.rs", 0)]);
        assert_eq!(
            check(&baseline, &current),
            vec![
                Violation::Reduced { path: "a.rs".into(), baseline: 2, current: 1 },
                Violation::Gone { path: "gone.rs".into() },
                Violation::Reduced { path: "z.rs".into(), baseline: 1, current: 0 },
            ]
        );
    }

    #[test]
    fn update_refuses_increase_or_new_file() {
        let baseline = map(&[("a.rs", 2)]);
        assert!(plan_update(Some(&baseline), &map(&[("a.rs", 3)])).is_err());
        assert!(plan_update(Some(&baseline), &map(&[("a.rs", 2), ("new.rs", 1)])).is_err());
    }

    #[test]
    fn update_accepts_decrease_and_drops_empty_and_gone() {
        let baseline = map(&[("a.rs", 2), ("b.rs", 3), ("gone.rs", 1)]);
        let current = map(&[("a.rs", 1), ("b.rs", 0), ("c.rs", 0)]);
        assert_eq!(plan_update(Some(&baseline), &current), Ok(map(&[("a.rs", 1)])));
    }

    #[test]
    fn update_bootstraps_without_a_baseline() {
        let current = map(&[("a.rs", 2), ("b.rs", 0)]);
        assert_eq!(plan_update(None, &current), Ok(map(&[("a.rs", 2)])));
    }

    #[test]
    fn totals_count_comments_and_files() {
        assert_eq!(totals(&map(&[("a.rs", 2), ("b.rs", 0), ("c.py", 1)])), (3, 2));
    }
}

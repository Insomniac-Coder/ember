//! The end-to-end test harness (Part XIX §5).
//!
//! Each `.em` file under `tests/milestones/` and `tests/run-pass/` carries its
//! expectations as `#$` annotations (`[TST-1]`, `[TST-2]`):
//!
//! ```text
//! #$ test: run-pass
//! #$ stdout: 5
//! #$ assert-c: !contains("ember_alloc")
//! ```
//!
//! `#$` is an ordinary comment to the compiler — `$` has no other meaning
//! anywhere in Ember — so an annotation can never be mistaken for source and
//! can never affect compilation.
//!
//! `[TST-0]` — annotations are read from the raw source text, not from the
//! token stream: a `compile-fail` test may be expected to fail at the lexer,
//! so its expectations must be readable even when the file does not tokenise.

use std::path::{Path, PathBuf};
use std::process::Command;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is compiler/ember_driver.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this package")
        .to_path_buf()
}

#[derive(Default, Debug)]
struct Expectations {
    kind: Option<String>,
    stdout: Option<String>,
    exit: Option<i32>,
    /// `!contains("…")` or `contains("…")` against the emitted C.
    assert_c: Vec<(bool, String)>,
}

fn parse_expectations(source: &str) -> Expectations {
    let mut expectations = Expectations::default();
    let mut collecting_stdout = false;
    let mut stdout_lines: Vec<String> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("#$") else {
            collecting_stdout = false;
            continue;
        };
        let rest = rest.trim();

        if let Some(value) = rest.strip_prefix("test:") {
            expectations.kind = Some(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("exit:") {
            expectations.exit = value.trim().parse().ok();
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("assert-c:") {
            let value = value.trim();
            let (expect_present, body) = match value.strip_prefix('!') {
                Some(body) => (false, body),
                None => (true, value),
            };
            if let Some(inner) = body
                .trim()
                .strip_prefix("contains(")
                .and_then(|s| s.strip_suffix(')'))
            {
                expectations.assert_c.push((expect_present, inner.trim_matches('"').to_string()));
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("stdout:") {
            collecting_stdout = true;
            let value = value.trim();
            if !value.is_empty() {
                stdout_lines.push(value.to_string());
            }
        } else if collecting_stdout {
            stdout_lines.push(rest.to_string());
        }
    }

    if !stdout_lines.is_empty() {
        expectations.stdout = Some(stdout_lines.join("\n"));
    }
    expectations
}

struct Run {
    stdout: String,
    stderr: String,
    exit: i32,
}

fn ember(args: &[&str], root: &Path) -> Run {
    let output = Command::new(EMBER)
        .args(args)
        .current_dir(root)
        .output()
        .expect("the ember binary runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

fn check_file(path: &Path, root: &Path) {
    let source = std::fs::read_to_string(path).expect("the test file is readable");
    let expectations = parse_expectations(&source);
    let relative = path.strip_prefix(root).unwrap_or(path).to_string_lossy().into_owned();
    let out_dir = std::env::temp_dir().join("ember-tests").join(
        path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
    );
    let out_dir_arg = out_dir.to_string_lossy().into_owned();

    // The emitted C, for `assert-c`.
    if !expectations.assert_c.is_empty() {
        let emitted = ember(&["build", &relative, "--emit", "c"], root);
        assert_eq!(emitted.exit, 0, "emitting C for {relative} failed:\n{}", emitted.stderr);
        for (expect_present, needle) in &expectations.assert_c {
            let present = emitted.stdout.contains(needle.as_str());
            assert_eq!(
                present, *expect_present,
                "{relative}: expected the emitted C {} {needle:?}\n--- emitted C ---\n{}",
                if *expect_present { "to contain" } else { "not to contain" },
                emitted.stdout
            );
        }
    }

    let run = ember(&["run", &relative, "--out-dir", &out_dir_arg], root);

    if let Some(expected) = &expectations.stdout {
        assert_eq!(
            run.stdout.trim_end(),
            expected.trim_end(),
            "{relative}: stdout differs\nstderr:\n{}",
            run.stderr
        );
    }
    let expected_exit = expectations.exit.unwrap_or(0);
    assert_eq!(
        run.exit, expected_exit,
        "{relative}: exit code differs\nstdout:\n{}\nstderr:\n{}",
        run.stdout, run.stderr
    );
}

fn check_directory(name: &str) -> usize {
    let root = workspace_root();
    let dir = root.join(name);
    if !dir.is_dir() {
        return 0;
    }
    let mut count = 0;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the test directory is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "em"))
        .collect();
    entries.sort();
    for path in entries {
        check_file(&path, &root);
        count += 1;
    }
    count
}

#[test]
fn milestones_pass() {
    let count = check_directory("tests/milestones");
    assert!(count > 0, "no milestone tests were found");
}

#[test]
fn run_pass_programs_pass() {
    check_directory("tests/run-pass");
}

#[test]
fn hello_world_builds_and_runs() {
    // Phase 0's exit criterion (Part XX.2).
    let root = workspace_root();
    let out_dir = std::env::temp_dir().join("ember-tests").join("hello");
    let run = ember(
        &["run", "examples/hello.em", "--out-dir", &out_dir.to_string_lossy()],
        &root,
    );
    assert_eq!(run.exit, 0, "hello.em failed:\n{}", run.stderr);
    assert_eq!(run.stdout.trim_end(), "hello, world");
}

#[test]
fn the_emitted_c_compiles_without_warnings() {
    // `[CG-C-1]` — warning-free under -std=c11 -Wall -Wextra.
    let root = workspace_root();
    let emitted = ember(
        &["build", "tests/milestones/m1_value_code_has_no_runtime_cost.em", "--emit", "c"],
        &root,
    );
    assert_eq!(emitted.exit, 0, "{}", emitted.stderr);
    // An unreferenced label is the failure this guards: lowering leaves
    // unreachable blocks behind, and every one would become a warning.
    for (index, line) in emitted.stdout.lines().enumerate() {
        if let Some(label) = line.trim().strip_suffix(": ;") {
            assert!(
                emitted.stdout.contains(&format!("goto {label};")),
                "line {}: label `{label}` is never jumped to, which is a warning under -Wall",
                index + 1
            );
        }
    }
}

#[test]
fn parse_expectations_reads_the_annotation_forms() {
    let source = "\
#$ test: run-pass
struct S:
    x: i32
#$ stdout: 5
#$ assert-c: !contains(\"ember_alloc\")
#$ exit: 0
";
    let parsed = parse_expectations(source);
    assert_eq!(parsed.kind.as_deref(), Some("run-pass"));
    assert_eq!(parsed.stdout.as_deref(), Some("5"));
    assert_eq!(parsed.exit, Some(0));
    assert_eq!(parsed.assert_c, vec![(false, "ember_alloc".to_string())]);
}

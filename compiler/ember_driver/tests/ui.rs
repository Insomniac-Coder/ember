//! Rendered diagnostic regression tests for `[DIA-13]` and `[PHIL-8a]`.
//!
//! A case lives at `tests/ui/borrow/<shape>/<name>.em`, with two companions:
//!
//! * `<name>.stderr` is the exact human diagnostic rendered for the rejected
//!   program;
//! * `<name>.fixed.em` is the program after applying the diagnostic's primary
//!   structural repair, and must compile.
//!
//! This suite deliberately discovers only committed cases. It does not claim
//! the whole catalogue is complete: shapes whose language feature or error
//! producer has not landed remain explicit implementation work in BACKLOG.md.

use std::path::{Path, PathBuf};
use std::process::Command;

use ember_diag::shapes::{shape_for, Shape};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels above this package")
        .to_path_buf()
}

fn normalize(text: &[u8]) -> String {
    String::from_utf8_lossy(text)
        .replace("\r\n", "\n")
        .replace('\\', "/")
}

fn check(path: &Path, root: &Path) -> (i32, String) {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let output = Command::new(EMBER)
        .args(["check", &relative])
        .current_dir(root)
        .output()
        .expect("the ember binary runs");
    (
        output.status.code().unwrap_or(-1),
        normalize(&output.stderr),
    )
}

fn error_codes(rendered: &str) -> Vec<&str> {
    rendered
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")
                .and_then(|rest| rest.split_once(']'))
                .map(|(code, _)| code)
        })
        .collect()
}

fn required_primary_fragments(shape: Shape) -> &'static [&'static str] {
    // These are the concrete constructs the normative catalogue requires a
    // primary suggestion to name. Exact rendering is pinned by `.stderr`;
    // this second check prevents an intentionally refreshed snapshot from
    // silently blessing help for a different shape.
    match shape {
        Shape::O1 => &["clone", "borrow"],
        Shape::O2 => &["mem.take", "mem.replace", "swap"],
        Shape::O3 => &["loop"],
        Shape::O4 => &["reassign", "destructure"],
        Shape::O5 => &["owned", "mem.take"],
        Shape::O6 => &["borrow"],
        Shape::O7 => &["mem.drop"],
        Shape::O8 => &["scope"],
        Shape::O9 => &["mem.take"],
        Shape::B1 => &["split_at_mut"],
        Shape::B2 => &["retain"],
        Shape::B3 => &["borrow"],
        Shape::B4 => &["single owner"],
        Shape::B5 => &["index", "Handle", "Weak"],
        Shape::B6 => &["@borrows"],
        Shape::B7 => &["owned value"],
        Shape::B8 => &["separate parameters"],
        Shape::B9 => &["owned fn"],
        Shape::B10 => &["local"],
        Shape::B11 => &["assert_disjoint"],
        Shape::B12 => &["owned copy"],
        Shape::B13 => &["separate parameters"],
        Shape::B14 => &["owned value", "separately", "@borrows"],
        Shape::B15 => &["expected callable signature", "supplied callable signature"],
        Shape::X1 => &["block"],
        Shape::S1 => &["access"],
        Shape::A1 => &["Arena", "arena", "alloc_nodrop", "scope"],
    }
}

fn primary_help(rendered: &str) -> &str {
    rendered
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("= help: "))
        .expect("a classified diagnostic has a primary help line")
}

fn cases(root: &Path) -> Vec<PathBuf> {
    let base = root.join("tests/ui/borrow");
    let fixed_suffix = format!(".fixed.{}", ember_branding::SOURCE_EXT);
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("the UI directory is readable") {
            let path = entry.expect("the UI entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext == ember_branding::SOURCE_EXT)
                && !path.to_string_lossy().ends_with(&fixed_suffix)
            {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn assert_no_orphan_companions(root: &Path, cases: &[PathBuf]) {
    let known: std::collections::BTreeSet<PathBuf> =
        cases.iter().map(|case| case.with_extension("")).collect();
    let base = root.join("tests/ui/borrow");
    let fixed_suffix = format!(".fixed.{}", ember_branding::SOURCE_EXT);
    let mut pending = vec![base];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("the UI directory is readable") {
            let path = entry.expect("the UI entry is readable").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let text = path.to_string_lossy();
            let stem = if text.ends_with(&fixed_suffix) {
                PathBuf::from(text.trim_end_matches(&fixed_suffix))
            } else if text.ends_with(".stderr") {
                path.with_extension("")
            } else {
                continue;
            };
            assert!(
                known.contains(&stem),
                "{} has no primary source case",
                path.display()
            );
        }
    }
}

#[test]
fn committed_borrow_snapshots_match_and_primary_fixes_compile() {
    let root = workspace_root();
    let cases = cases(&root);
    assert!(!cases.is_empty(), "no tests/ui/borrow cases were found");
    assert_no_orphan_companions(&root, &cases);

    for before in cases {
        let stem = before.with_extension("");
        let snapshot = stem.with_extension("stderr");
        let fixed = stem.with_extension(format!("fixed.{}", ember_branding::SOURCE_EXT));
        assert!(
            snapshot.is_file(),
            "{} has no .stderr snapshot",
            before.display()
        );
        assert!(
            fixed.is_file(),
            "{} has no fixed-source companion",
            before.display()
        );

        let expected = std::fs::read(&snapshot).expect("the snapshot is readable");
        let expected = normalize(&expected);
        let (before_exit, actual) = check(&before, &root);
        assert_ne!(before_exit, 0, "{} unexpectedly compiled", before.display());
        assert_eq!(actual, expected, "{} diagnostic changed", before.display());

        let codes = error_codes(&actual);
        assert_eq!(
            codes.len(),
            1,
            "{} must isolate exactly one rendered error code, found {codes:?}",
            before.display()
        );
        let code = ember_diag::codes::lookup(codes[0])
            .unwrap_or_else(|| panic!("{} is not registered", codes[0]));
        let actual_shape =
            shape_for(code).unwrap_or_else(|| panic!("{} has no DIA-7a shape", codes[0]));
        let expected_shape = before
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .expect("a case has a shape directory");
        assert_eq!(
            actual_shape.to_string(),
            expected_shape,
            "{} is filed under the wrong shape",
            before.display()
        );

        let help = primary_help(&actual);
        let required = required_primary_fragments(actual_shape);
        assert!(
            required.iter().any(|fragment| help.contains(fragment)),
            "{} primary help {help:?} names none of the required constructs {required:?}",
            before.display()
        );

        let (fixed_exit, fixed_stderr) = check(&fixed, &root);
        assert_eq!(
            fixed_exit,
            0,
            "{} primary fix does not compile:\n{fixed_stderr}",
            fixed.display()
        );
    }
}

#[test]
fn committed_basic_name_suggestion_matches_and_fixed_source_compiles() {
    let root = workspace_root();
    let before = root.join("tests/ui/basic/N1/local_name_typo.em");
    let snapshot = before.with_extension("stderr");
    let fixed = root.join("tests/ui/basic/N1/local_name_typo.fixed.em");

    let expected = std::fs::read(&snapshot).expect("the snapshot is readable");
    let expected = normalize(&expected);
    let (before_exit, actual) = check(&before, &root);
    assert_ne!(before_exit, 0, "{} unexpectedly compiled", before.display());
    assert_eq!(actual, expected, "{} diagnostic changed", before.display());
    assert_eq!(
        error_codes(&actual),
        ["E1010"],
        "{} must isolate the unknown-name diagnostic",
        before.display()
    );
    assert!(
        actual.contains("did you mean `count`?"),
        "N1 should suggest the visible binding with its exact spelling:\n{actual}"
    );

    let (fixed_exit, fixed_stderr) = check(&fixed, &root);
    assert_eq!(
        fixed_exit,
        0,
        "{} does not compile:\n{fixed_stderr}",
        fixed.display()
    );
}

#[test]
fn committed_struct_field_name_suggestion_matches_and_fixed_source_compiles() {
    let root = workspace_root();
    let before = root.join("tests/ui/basic/N1/struct_field_typo.em");
    let snapshot = before.with_extension("stderr");
    let fixed = root.join("tests/ui/basic/N1/struct_field_typo.fixed.em");

    let expected = std::fs::read(&snapshot).expect("the snapshot is readable");
    let expected = normalize(&expected);
    let (before_exit, actual) = check(&before, &root);
    assert_ne!(before_exit, 0, "{} unexpectedly compiled", before.display());
    assert_eq!(actual, expected, "{} diagnostic changed", before.display());
    assert_eq!(
        error_codes(&actual),
        ["E1010"],
        "{} must classify the missing field as an unresolved name",
        before.display()
    );
    assert!(
        actual.contains("did you mean `x`?"),
        "N1 should suggest the visible struct field with its exact spelling:\n{actual}"
    );

    let (fixed_exit, fixed_stderr) = check(&fixed, &root);
    assert_eq!(
        fixed_exit,
        0,
        "{} does not compile:\n{fixed_stderr}",
        fixed.display()
    );
}

#[test]
fn committed_inherited_class_field_name_suggestion_matches_and_fixed_source_compiles() {
    let root = workspace_root();
    let before = root.join("tests/ui/basic/N1/inherited_class_field_typo.em");
    let snapshot = before.with_extension("stderr");
    let fixed = root.join("tests/ui/basic/N1/inherited_class_field_typo.fixed.em");

    let expected = std::fs::read(&snapshot).expect("the snapshot is readable");
    let expected = normalize(&expected);
    let (before_exit, actual) = check(&before, &root);
    assert_ne!(before_exit, 0, "{} unexpectedly compiled", before.display());
    assert_eq!(actual, expected, "{} diagnostic changed", before.display());
    assert_eq!(
        error_codes(&actual),
        ["E1010"],
        "{} must classify the missing field as an unresolved name",
        before.display()
    );
    assert!(
        actual.contains("did you mean `value`?"),
        "N1 should suggest the inherited field with its exact spelling:\n{actual}"
    );

    let (fixed_exit, fixed_stderr) = check(&fixed, &root);
    assert_eq!(
        fixed_exit,
        0,
        "{} does not compile:\n{fixed_stderr}",
        fixed.display()
    );
}

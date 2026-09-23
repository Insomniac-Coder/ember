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
use std::time::{SystemTime, UNIX_EPOCH};

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
    let before = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("local_name_typo"));
    let snapshot = before.with_extension("stderr");
    let fixed = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("local_name_typo.fixed"));

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
    let before = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("struct_field_typo"));
    let snapshot = before.with_extension("stderr");
    let fixed = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("struct_field_typo.fixed"));

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
    let before = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("inherited_class_field_typo"));
    let snapshot = before.with_extension("stderr");
    let fixed = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file(
            "inherited_class_field_typo.fixed",
        ));

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

#[test]
fn committed_module_item_name_suggestion_matches_and_fixed_source_compiles() {
    let root = workspace_root();
    let before = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("module_item_typo"));
    let snapshot = before.with_extension("stderr");
    let fixed = root
        .join("tests/ui/basic/N1")
        .join(ember_branding::source_file("module_item_typo.fixed"));

    let expected = std::fs::read(&snapshot).expect("the snapshot is readable");
    let expected = normalize(&expected);
    let (before_exit, actual) = check(&before, &root);
    assert_ne!(before_exit, 0, "{} unexpectedly compiled", before.display());
    assert_eq!(actual, expected, "{} diagnostic changed", before.display());
    assert_eq!(
        error_codes(&actual),
        ["E1010"],
        "{} must isolate the unresolved module item",
        before.display()
    );
    assert!(
        actual.contains("did you mean `calculate`?"),
        "N1 should suggest the visible module item with its exact spelling:\n{actual}"
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
fn qualified_name_suggestion_preserves_namespace_and_targets_leaf() {
    let workspace = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the Unix epoch")
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "ember-n1-qualified-name-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(package.join("support"))
        .expect("support module directory is creatable");
    std::fs::write(
        package.join(ember_branding::source_file("support/io")),
        "pub fn print(value: i32) -> i32:\n    return value\n",
    )
    .expect("namespace module is writable");
    let main = ember_branding::source_file("main");
    let main_path = package.join(&main);
    let typo = "import support.io as io\n\nfn main():\n    io.pritn(1)\n";
    std::fs::write(&main_path, typo).expect("misspelled source is writable");

    let check = |json, std_root: &Path| {
        let mut command = Command::new(EMBER);
        command.args(["check", &main]);
        if json {
            command.arg("--json");
        }
        command
            .current_dir(&package)
            .env(ember_branding::std_path_var(), std_root)
            .output()
            .expect("the Ember compiler runs")
    };
    let before = check(false, &workspace.join("std"));
    assert!(
        !before.status.success(),
        "the unresolved qualified name must remain rejected"
    );
    let stderr = normalize(&before.stderr);
    assert_eq!(error_codes(&stderr), ["E1010"]);
    assert!(
        stderr.contains("did you mean `io.print`?"),
        "N1 should render the source-qualified candidate while preserving its alias:\n{stderr}"
    );
    let lines = stderr.lines().collect::<Vec<_>>();
    let source_line = lines
        .iter()
        .position(|line| line.contains("io.pritn(1)"))
        .expect("the diagnostic includes the misspelled source line");
    let marker = lines[source_line + 1..]
        .iter()
        .find(|line| line.contains('^'))
        .expect("the diagnostic highlights the unresolved final segment");
    assert_eq!(
        marker.matches('^').count(),
        "pritn".len(),
        "the primary span must cover only the unresolved final identifier:\n{stderr}"
    );
    let json = check(true, &workspace.join("std"));
    let json = String::from_utf8_lossy(&json.stdout);
    let edit_start = typo.find("pritn").expect("the typo occurs in the source");
    let edit_end = edit_start + "pritn".len();
    assert!(
        json.contains("\"replacement\":\"print\""),
        "the edit should replace only the final token:\n{json}"
    );
    assert!(
        json.contains(&format!("\"byte_start\":{edit_start}")),
        "the edit should start at `pritn`:\n{json}"
    );
    assert!(
        json.contains(&format!("\"byte_end\":{edit_end}")),
        "the edit should end after `pritn`:\n{json}"
    );

    let fixed = typo.replace("io.pritn", "io.print");
    std::fs::write(&main_path, fixed).expect("corrected source is writable");
    let after = check(false, &workspace.join("std"));
    assert!(
        after.status.success(),
        "the token-local correction must compile without rewriting the alias:\n{}",
        normalize(&after.stderr)
    );

    let foreign_std = package.join("separate-std");
    std::fs::create_dir_all(&foreign_std).expect("separate standard package is creatable");
    std::fs::write(
        foreign_std.join(ember_branding::source_file("hidden")),
        "pub(package) fn print(value: i32) -> i32:\n    return value\n",
    )
    .expect("package-private declaration is writable");
    std::fs::write(
        &main_path,
        "import std.hidden as foreign\n\nfn main():\n    foreign.pritn(1)\n",
    )
    .expect("foreign-package typo source is writable");
    let foreign = check(false, &foreign_std);
    assert!(
        !foreign.status.success(),
        "the unknown qualified item must remain rejected"
    );
    let foreign_stderr = normalize(&foreign.stderr);
    assert_eq!(error_codes(&foreign_stderr), ["E1010"]);
    assert!(
        !foreign_stderr.contains("did you mean"),
        "a `pub(package)` name from another package must not leak as a suggestion:\n{foreign_stderr}"
    );
    std::fs::remove_dir_all(&package).expect("only the test package is removed");
}

#[test]
fn qualified_n1_ranking_is_stable_and_bad_prefixes_do_not_search_members() {
    let root = workspace_root();
    let candidates = root
        .join("tests/conformance/DIA-12/reject_qualified_top_three_visible_candidates")
        .with_extension(ember_branding::SOURCE_EXT);
    let (exit, first) = check(&candidates, &root);
    assert_ne!(exit, 0, "the misspelled qualified item must be rejected");
    assert_eq!(error_codes(&first), ["E1010"]);
    let ordered = [
        "did you mean `picks.prin`?",
        "did you mean `picks.print`?",
        "did you mean `picks.prit`?",
    ];
    let positions = ordered.map(|suggestion| {
        first
            .find(suggestion)
            .unwrap_or_else(|| panic!("missing N1 candidate {suggestion:?}:\n{first}"))
    });
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    for _ in 0..3 {
        let (repeat_exit, repeated) = check(&candidates, &root);
        assert_eq!(repeat_exit, exit);
        assert_eq!(
            repeated, first,
            "qualified N1 ordering must be deterministic"
        );
    }

    let bad_prefix = root
        .join("tests/conformance/DIA-12/reject_qualified_prefix_typo_is_not_searched")
        .with_extension(ember_branding::SOURCE_EXT);
    let (prefix_exit, prefix_stderr) = check(&bad_prefix, &root);
    assert_ne!(prefix_exit, 0);
    assert_eq!(error_codes(&prefix_stderr), ["E1010"]);
    assert!(
        !prefix_stderr.contains("did you mean"),
        "a failed prefix must not search the final segment:\n{prefix_stderr}"
    );
    let lines = prefix_stderr.lines().collect::<Vec<_>>();
    let source_line = lines
        .iter()
        .position(|line| line.contains("ioo.print(1)"))
        .expect("the unresolved-prefix source line is rendered");
    let marker = lines[source_line + 1..]
        .iter()
        .find(|line| line.contains('^'))
        .expect("the unresolved prefix is highlighted");
    assert_eq!(marker.matches('^').count(), "ioo".len());
}

#[test]
fn unknown_type_suggests_visible_import_alias_and_corrected_type_compiles() {
    let workspace = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the Unix epoch")
        .as_nanos();
    let package = std::env::temp_dir().join(format!(
        "ember-n1-type-position-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&package).expect("temporary package directory is creatable");
    let model = ember_branding::source_file("model");
    std::fs::write(
        package.join(model),
        "pub struct Point:\n    x: i32\n\npub struct Vector[T]:\n    value: T\n",
    )
    .expect("candidate module is writable");
    let main = ember_branding::source_file("main");
    let main_path = package.join(&main);
    std::fs::write(
        &main_path,
        "from model import Point as Dot, Vector as Vect\n\nfn takes(value: Dto):\n    pass\n\nfn takes_vector(value: Vetc[i32]):\n    pass\n\nfn main():\n    pass\n",
    )
    .expect("misspelled type source is writable");

    let check = || {
        Command::new(EMBER)
            .args(["check", &main])
            .current_dir(&package)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs")
    };
    let before = check();
    assert!(
        !before.status.success(),
        "the misspelled type must be rejected"
    );
    let stderr = normalize(&before.stderr);
    assert_eq!(error_codes(&stderr), ["E1010", "E1010"]);
    assert!(
        stderr.contains("did you mean `Dot`?") && stderr.contains("did you mean `Vect`?"),
        "N1 should suggest visible aliases in plain and generic type positions:\n{stderr}"
    );

    std::fs::write(
        &main_path,
        "from model import Point as Dot, Vector as Vect\n\nfn takes(value: Dot):\n    pass\n\nfn takes_vector(value: Vect[i32]):\n    pass\n\nfn main():\n    pass\n",
    )
    .expect("corrected type source is writable");
    let fixed = check();
    assert!(
        fixed.status.success(),
        "the corrected type does not compile:\n{}",
        normalize(&fixed.stderr)
    );
    std::fs::remove_dir_all(&package).expect("only the test package is removed");
}

#[test]
fn cross_file_name_suggestions_ignore_file_and_import_discovery_order() {
    let workspace = workspace_root();
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the Unix epoch")
        .as_nanos();
    let scratch = std::env::temp_dir().join(format!(
        "ember-n1-cross-file-{}-{nonce}",
        std::process::id()
    ));
    let cases = [
        (
            "io-first",
            ["io", "text"],
            "from text import print\nfrom io import print as prtin",
        ),
        (
            "text-first",
            ["text", "io"],
            "from io import print as prtin\nfrom text import print",
        ),
    ];
    let expected = vec![
        "did you mean `prtin`?".to_owned(),
        "did you mean `print`?".to_owned(),
    ];

    for (label, creation_order, imports) in cases {
        let package = scratch.join(label);
        std::fs::create_dir_all(&package).expect("temporary package directory is creatable");
        for module in creation_order {
            std::fs::write(
                package.join(ember_branding::source_file(module)),
                "pub fn print(value: i32) -> i32:\n    return value\n",
            )
            .expect("candidate module is writable");
        }
        let main = ember_branding::source_file("main");
        std::fs::write(
            package.join(&main),
            format!("{imports}\n\nfn main():\n    pritn(1)\n"),
        )
        .expect("entry module is writable");

        let mut previous = None;
        for _ in 0..2 {
            let output = Command::new(EMBER)
                .args(["check", &main])
                .current_dir(&package)
                .env(ember_branding::std_path_var(), workspace.join("std"))
                .output()
                .expect("the Ember compiler runs");
            assert!(
                !output.status.success(),
                "the misspelled call must be rejected"
            );
            let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
            let helps: Vec<_> = stderr
                .lines()
                .filter_map(|line| line.trim_start().strip_prefix("= help: "))
                .map(str::to_owned)
                .collect();
            assert_eq!(helps, expected, "{label}: unexpected N1 ordering\n{stderr}");
            if let Some(previous) = &previous {
                assert_eq!(
                    &helps, previous,
                    "{label}: repeated check changed N1 ordering"
                );
            }
            previous = Some(helps);
        }
    }

    std::fs::remove_dir_all(&scratch).expect("only the test's temporary packages are removed");
}

#[test]
fn a_returned_view_of_a_local_is_one_error() {
    // D-190, `[DIA-14]` — the drop of `tmp` at the return is the escape
    // `E3060` reports, not a second, `E3021` write while borrowed.
    let root = workspace_root();
    let path = root
        .join("tests/conformance/LT-3/reject_a_returned_view_of_a_local")
        .with_extension(ember_branding::SOURCE_EXT);
    let (exit, stderr) = check(&path, &root);
    assert_ne!(exit, 0, "the escaping view must be rejected");
    assert_eq!(error_codes(&stderr), ["E3060"], "{stderr}");
}

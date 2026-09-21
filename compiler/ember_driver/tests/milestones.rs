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
use std::time::{SystemTime, UNIX_EPOCH};

use ember_build::interface::{CallableParameterMode, ModuleInterfaceArtifact};
use ember_branding::{MANIFEST, SOURCE_EXT};

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
    /// Profiles in which this program must have the same specified result.
    /// Empty means the ordinary `debug` run. A rule with an explicit
    /// every-profile quantifier must list all three rather than relying on an
    /// implementation comment that says the lowering has no profile input.
    profiles: Vec<String>,
    /// `!contains("…")` or `contains("…")` against the emitted C.
    assert_c: Vec<(bool, String)>,
    /// `#$ assert-c-count: contains("…") == N` — the needle occurs exactly
    /// `N` times in the emitted C.
    assert_c_count: Vec<(String, usize)>,
    /// `#$ assert-c-order: "a" then "b"` — both appear in the emitted C, and
    /// the first before the second.
    ///
    /// `assert-c`'s needle is one line of literal text, so it can say what the
    /// backend emitted but not in what **order**. Some rules are entirely about
    /// order: `[CELL-1]` requires `set` to store the new value *before*
    /// dropping the old one, and `[OWN-5]` requires an assignment to do the
    /// opposite. Both are invisible in a program's output — the difference only
    /// shows if a destructor re-enters the value being replaced, which needs a
    /// back-pointer no safe program can build — so without this the two rules
    /// could only be checked by reading the C by hand, once, and would silently
    /// stop holding the next time either path was touched.
    assert_c_order: Vec<(String, String)>,
    /// A `run-fail` test: the program must panic, and the message must contain
    /// this text.
    panics: Option<String>,
    /// `#$ error[EXXXX]: text` — a diagnostic the compiler must produce.
    ///
    /// `[TST-1]` also ties an annotation to the line its primary span starts
    /// on. That form waits for the conformance suite; here the code and a
    /// substring of the message are matched anywhere in the output, which is
    /// what a `compile-fail` test in `tests/` needs.
    errors: Vec<(String, String)>,
    /// Ordered `= help:` substrings required from a diagnostic.
    helps: Vec<String>,
    /// Text that must not occur in any `= help:` line. This pins negative
    /// suggestion policy such as `[CELL-10]` without matching source comments.
    forbidden_helps: Vec<String>,
    /// `#$ warning[WXXXX]: text` / `#$ warning[LXXXX]: text` — a warning or
    /// lint the compiler must produce. These were parsed by nobody until
    /// 2026-09-08, so a file could assert any warning at all and pass.
    warnings: Vec<(String, String)>,
}

fn parse_expectations(source: &str) -> Expectations {
    let mut expectations = Expectations::default();
    let mut collecting_stdout = false;
    let mut stdout_lines: Vec<String> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim_start();
        // An annotation may open the line, or trail the code it is about:
        //
        //     a.push(2)     #$ error[E3021]: `a` is borrowed here
        //
        // The trailing form is the one nearly every `compile-fail` case uses,
        // because it puts the expected diagnostic on the line that provokes
        // it. It was **not recognised** until now: only the leading form was,
        // so every trailing `error[...]` was read as ordinary comment text and
        // asserted nothing. The suites still passed, because a `compile-fail`
        // test with no parsed expectations falls back to "compilation must
        // fail" — and it did fail, for whatever reason, related or not.
        // `#` opens a comment in Ember, so the text from `#$` onward is
        // already inert to the compiler either way.
        let rest = match trimmed.strip_prefix("#$") {
            Some(rest) => rest,
            None => match line.find("#$") {
                Some(at) if line[..at].ends_with(char::is_whitespace) => &line[at + 2..],
                _ => {
                    collecting_stdout = false;
                    continue;
                }
            },
        };
        let rest = rest.trim();

        if let Some(value) = rest.strip_prefix("test:") {
            expectations.kind = Some(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("profiles:") {
            expectations.profiles = value
                .split(',')
                .map(str::trim)
                .filter(|profile| !profile.is_empty())
                .map(str::to_string)
                .collect();
            for profile in &expectations.profiles {
                assert!(
                    matches!(profile.as_str(), "debug" | "release" | "shipping"),
                    "unknown test profile {profile:?}"
                );
            }
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
                expectations
                    .assert_c
                    .push((expect_present, inner.trim_matches('"').to_string()));
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("assert-c-count:") {
            if let Some((body, expected)) = value.rsplit_once("==") {
                if let (Some(inner), Ok(expected)) = (
                    body.trim()
                        .strip_prefix("contains(")
                        .and_then(|s| s.strip_suffix(')')),
                    expected.trim().parse::<usize>(),
                ) {
                    expectations
                        .assert_c_count
                        .push((inner.trim_matches('"').to_string(), expected));
                }
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("assert-c-order:") {
            // `"first" then "second"`. Split on the keyword rather than on the
            // quotes, so either needle may contain one.
            if let Some((before, after)) = value.split_once(" then ") {
                let first = before.trim().trim_matches('"').to_string();
                let second = after.trim().trim_matches('"').to_string();
                if !first.is_empty() && !second.is_empty() {
                    expectations.assert_c_order.push((first, second));
                }
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("error[") {
            if let Some((code, message)) = value.split_once("]:") {
                expectations
                    .errors
                    .push((code.trim().to_string(), message.trim().to_string()));
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("warning[") {
            if let Some((code, message)) = value.split_once("]:") {
                expectations
                    .warnings
                    .push((code.trim().to_string(), message.trim().to_string()));
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("help:") {
            expectations.helps.push(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("not-help:") {
            expectations.forbidden_helps.push(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("panics:") {
            expectations.panics = Some(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("stdout:") {
            collecting_stdout = true;
            let value = value.trim();
            if !value.is_empty() {
                stdout_lines.push(value.to_string());
            }
        } else if collecting_stdout {
            stdout_lines.push(rest.to_string());
        } else if !rest.is_empty() && !rest.starts_with("rules:") && !rest.starts_with("note") {
            // An unrecognised key used to fall through in silence, so
            // `#$ panic:` for `#$ panics:` asserted nothing and the test
            // passed. Every annotation is now either understood or refused.
            panic!(
                "unrecognised `#$` annotation: {rest:?}
  known keys:                  test, profiles, exit, assert-c, error[…], warning[…], help, not-help, panics, stdout, rules, note"
            );
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

/// The compiler's own rendering of a diagnostic, with the **echoed source
/// lines removed**.
///
/// This matters more than it looks. A `compile-fail` case writes its
/// expectation on the line that provokes it:
///
///     p.title = 7     #$ error[E1050]: `title` is read-only outside its module
///
/// and the diagnostic renderer prints that source line back inside the error
/// it produces. So `stderr.contains("E1050")` was satisfied by the test's own
/// annotation being echoed — **every trailing expectation asserted itself**,
/// and a case expecting a code the compiler never emits passed. Found by
/// writing one whose expected code was wrong (`E1021` for what is `E1050`)
/// and watching it go green.
///
/// Only the **numbered** lines are the echo: `18 |     match c:` is the
/// programmer's source and may hold the annotation, while `   | ^^^^ label`
/// beneath it carries the compiler's own primary label and must be kept — a
/// filter that drops both throws away the words half the expectations are
/// written against.
fn without_source_echo(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            let digits = t.trim_start_matches(|c: char| c.is_ascii_digit());
            // A numbered gutter, and nothing else, is a line of source.
            !(digits.len() < t.len() && digits.trim_start().starts_with('|'))
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        )
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

fn temporary_directory(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock is after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "ember-{label}-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&path).expect("temporary directory is creatable");
    path
}

#[test]
fn dynamic_access_safety_side_table_is_written() {
    let root = workspace_root();
    let source = format!(
        "tests/run-pass/class_mut_method_access.{SOURCE_EXT}"
    );
    let out_dir = std::env::temp_dir().join(format!(
        "ember-safety-side-table-{}",
        std::process::id()
    ));
    let run = ember(
        &[
            "build",
            &source,
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        &root,
    );
    assert_eq!(run.exit, 0, "build failed:\n{}", run.stderr);

    let side_table = out_dir
        .join("debug")
        .join("inspect")
        .join("class_mut_method_access.safety.json");
    let json = std::fs::read_to_string(&side_table)
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
    assert!(json.contains("\"schema\":1"), "missing side-table schema: {json}");
    assert!(json.contains("\"kind\":\"Aliasing\""), "missing access kind: {json}");
    assert!(json.contains("\"function\":\"bump\""), "missing function identity: {json}");
    assert!(
        json.contains("\"status\":\"emitted\""),
        "missing emitted status: {json}"
    );

    let side_table_arg = side_table.to_string_lossy().into_owned();
    let report = ember(&["inspect", "--safety", &side_table_arg], &root);
    assert_eq!(report.exit, 0, "inspect failed:\n{}", report.stderr);
    assert!(
        report.stdout.contains("Aliasing 1"),
        "inspect did not report the emitted check:\n{}",
        report.stdout
    );
    let json_report = ember(
        &["inspect", "--safety", "--json", &side_table_arg],
        &root,
    );
    assert_eq!(json_report.exit, 0, "JSON inspect failed:\n{}", json_report.stderr);
    assert!(
        json_report.stdout.contains("\"status\":\"emitted\""),
        "JSON inspect omitted the emitted record:\n{}",
        json_report.stdout
    );
    let filtered = ember(
        &["inspect", "--safety", "--elided-only", &side_table_arg],
        &root,
    );
    assert_eq!(filtered.exit, 0, "elided-only inspect failed:\n{}", filtered.stderr);
    assert!(
        filtered.stdout.contains("none"),
        "elided-only inspect should be empty until elision exists:\n{}",
        filtered.stdout
    );
    let function_filtered = ember(
        &[
            "inspect",
            "--safety",
            "--function",
            "bump",
            &side_table_arg,
        ],
        &root,
    );
    assert_eq!(
        function_filtered.exit, 0,
        "function-filtered inspect failed:\n{}",
        function_filtered.stderr
    );
    assert!(
        function_filtered.stdout.contains("Function: bump")
            && function_filtered.stdout.contains("bump"),
        "function-filtered inspect omitted bump:\n{}",
        function_filtered.stdout
    );
    let function_json = ember(
        &[
            "inspect",
            "--safety",
            "--json",
            "--function",
            "bump",
            &side_table_arg,
        ],
        &root,
    );
    assert_eq!(function_json.exit, 0, "JSON function filter failed:\n{}", function_json.stderr);
    assert!(
        function_json.stdout.contains("\"function\":\"bump\""),
        "JSON function filter omitted bump:\n{}",
        function_json.stdout
    );
}

/// `[EXC-8]`–`[EXC-14]` — a copied class handle prevents static unique-handle
/// elision, but the receiver itself is stable across this counted loop. The
/// compiler therefore emits one checked loop-level interval, reports its
/// proof, and leaves the callee's independent receiver access alone.
#[test]
fn stable_class_loop_access_is_reported_as_hoisted() {
    let root = workspace_root();
    let source = format!(
        "tests/conformance/EXC-8/accept_stable_class_receiver_loop.{SOURCE_EXT}"
    );
    let out_dir = std::env::temp_dir().join(format!(
        "ember-hoisted-loop-safety-{}",
        std::process::id()
    ));
    let run = ember(
        &[
            "build",
            &source,
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        &root,
    );
    assert_eq!(run.exit, 0, "build failed:\n{}", run.stderr);

    let side_table = out_dir
        .join("debug")
        .join("inspect")
        .join("accept_stable_class_receiver_loop.safety.json");
    let json = std::fs::read_to_string(&side_table)
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
    assert!(
        json.contains("\"classification\":\"DYNAMIC_HOISTED_LOOP\""),
        "stable loop was not classified as hoisted:\n{json}"
    );
    assert!(
        json.contains("\"check_site\":\"preheader\""),
        "hoisted loop lacks its preheader check record:\n{json}"
    );
    assert!(
        json.contains("\"protected_interval\":\"loop\""),
        "hoisted loop lacks its protected interval record:\n{json}"
    );
    assert!(
        json.contains("\"proof\":\"stable_receiver_direct_call\""),
        "hoisted loop lacks its proof record:\n{json}"
    );

    let c_path = out_dir
        .join("debug")
        .join("c")
        .join("accept_stable_class_receiver_loop.c");
    let c = std::fs::read_to_string(&c_path)
        .unwrap_or_else(|error| panic!("{}: {error}", c_path.display()));
    let main_start = c.find("void em_main(void)").expect("main is emitted");
    let bump_start = c.rfind("void em_Counter_bump(").expect("method is emitted");
    let main_c = &c[main_start..bump_start];
    assert_eq!(
        main_c.matches("ember_access_begin_write").count(),
        1,
        "main must have one preheader write check:\n{main_c}"
    );
    assert_eq!(
        main_c.matches("ember_access_end_write").count(),
        1,
        "main must have one postheader write release:\n{main_c}"
    );

    let side_table_arg = side_table.to_string_lossy().into_owned();
    let report = ember(&["inspect", "--safety", &side_table_arg], &root);
    assert_eq!(report.exit, 0, "inspect failed:\n{}", report.stderr);
    assert!(
        report.stdout.contains("DYNAMIC_HOISTED_LOOP")
            && report.stdout.contains("proof: stable_receiver_direct_call"),
        "inspect did not identify the hoisted proof:\n{}",
        report.stdout
    );
}

/// `[EXC-9]` / `[EXC-13]` / `[TST-15]` — the two proved loop forms hoist
/// exactly one caller check. Publishing a handle, dynamic dispatch, and two
/// receiver identities each deliberately retain their individual checks.
#[test]
fn loop_access_hoisting_requires_the_complete_local_proof() {
    let root = workspace_root();
    let cases = [
        (
            "accept_invariant_local_receiver_loop",
            "DYNAMIC_HOISTED_LOOP",
            1usize,
        ),
        (
            "accept_escaping_receiver_loop_keeps_dynamic_check",
            "DYNAMIC_PER_ACCESS",
            1usize,
        ),
        (
            "accept_virtual_receiver_loop_keeps_dynamic_check",
            "DYNAMIC_PER_ACCESS",
            1usize,
        ),
        (
            "accept_distinct_receivers_loop_keeps_dynamic_checks",
            "DYNAMIC_PER_ACCESS",
            2usize,
        ),
    ];

    for (name, classification, expected) in cases {
        let source = format!("tests/conformance/EXC-8/{name}.{SOURCE_EXT}");
        let out_dir = temporary_directory(&format!("ember-loop-access-{name}"));
        let run = ember(
            &[
                "build",
                &source,
                "--out-dir",
                &out_dir.to_string_lossy(),
            ],
            &root,
        );
        assert_eq!(run.exit, 0, "{name} did not build:\n{}", run.stderr);

        let side_table = out_dir
            .join("debug")
            .join("inspect")
            .join(format!("{name}.safety.json"));
        let value: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&side_table)
                .unwrap_or_else(|error| panic!("{}: {error}", side_table.display())),
        )
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
        let observed = value
            .get("checks")
            .and_then(serde_json::Value::as_array)
            .expect("safety side table has checks")
            .iter()
            .filter(|check| {
                check.get("function").and_then(serde_json::Value::as_str) == Some("main")
                    && check.get("classification").and_then(serde_json::Value::as_str)
                        == Some(classification)
            })
            .count();
        assert_eq!(
            observed, expected,
            "{name} should have {expected} main {classification} check(s): {value}"
        );
        let wrong = if classification == "DYNAMIC_HOISTED_LOOP" {
            "DYNAMIC_PER_ACCESS"
        } else {
            "DYNAMIC_HOISTED_LOOP"
        };
        assert!(
            !value
                .get("checks")
                .and_then(serde_json::Value::as_array)
                .expect("safety side table has checks")
                .iter()
                .any(|check| {
                    check.get("function").and_then(serde_json::Value::as_str) == Some("main")
                        && check.get("classification").and_then(serde_json::Value::as_str)
                            == Some(wrong)
                }),
            "{name} has an unsafe mixed loop classification: {value}"
        );
    }
}

#[test]
fn static_access_elision_is_recorded_in_the_safety_side_table() {
    let root = workspace_root();
    let out_dir = std::env::temp_dir().join(format!(
        "ember-static-elision-side-table-{}",
        std::process::id()
    ));
    let run = ember(
        &[
            "build",
            &format!("tests/run-pass/class_field_mut_method.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        &root,
    );
    assert_eq!(run.exit, 0, "build failed:\n{}", run.stderr);

    let side_table = out_dir
        .join("debug")
        .join("inspect")
        .join("class_field_mut_method.safety.json");
    let json = std::fs::read_to_string(&side_table)
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
    assert!(
        json.contains("\"reason\":\"unique_handle\""),
        "missing unique-handle elision: {json}"
    );
    assert!(
        json.contains("\"status\":\"elided\""),
        "missing elided status: {json}"
    );
}

#[test]
fn cycle_inspection_reports_edge_kinds_and_shortest_cycle() {
    let root = workspace_root();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("cycle_inspection.{SOURCE_EXT}"));
    let fixture = fixture
        .strip_prefix(&root)
        .unwrap_or_else(|error| {
            panic!(
                "{} is outside {}: {error}",
                fixture.display(),
                root.display()
            )
        })
        .to_string_lossy()
        .into_owned();
    let report = ember(
        &[
            "inspect",
            "--cycle",
            &fixture,
        ],
        &root,
    );
    assert_eq!(report.exit, 0, "cycle inspection failed:\n{}", report.stderr);
    assert!(
        report.stdout.contains("strong Root.child: Child -> Child"),
        "cycle inspection omitted a direct strong edge:\n{}",
        report.stdout
    );
    assert!(
        report.stdout.contains("weak Child.root: Weak[Root] -> Root"),
        "cycle inspection omitted a weak edge:\n{}",
        report.stdout
    );
    assert!(
        report.stdout.contains("strong Child.leaf: Shared[Leaf] -> Leaf"),
        "cycle inspection leaked compiler-private generic spelling:\n{}",
        report.stdout
    );
    assert!(
        report
            .stdout
            .contains("Root.child -> Child.leaf -> Leaf.root -> Root"),
        "cycle inspection omitted the shortest static cycle:\n{}",
        report.stdout
    );

    let json = ember(
        &[
            "inspect",
            "--cycle",
            "--json",
            &fixture,
        ],
        &root,
    );
    assert_eq!(json.exit, 0, "JSON cycle inspection failed:\n{}", json.stderr);
    assert!(
        json.stdout.contains("\"kind\":\"weak\"")
            && json.stdout.contains("\"shortest_cycles\""),
        "JSON cycle inspection omitted graph facts:\n{}",
        json.stdout
    );
}

#[test]
fn cycle_inspection_uses_a_package_directory_as_its_analysis_root() {
    let root = workspace_root();
    let package = temporary_directory("cycle-package-root");
    let source = package.join("src");
    std::fs::create_dir_all(&source).expect("package source directory is creatable");
    std::fs::write(
        package.join(MANIFEST),
        "[package]\nname = \"cycle_root\"\nkind = \"bin\"\n",
    )
    .expect("package manifest is writable");
    std::fs::write(
        source.join(ember_branding::source_file("main")),
        "import scene\n\nfn main():\n    return\n",
    )
        .expect("package root source is writable");
    std::fs::write(
        source.join(ember_branding::source_file("scene")),
        "pub class Node:\n    next: Node\n",
    )
    .expect("imported module is writable");

    let package_arg = package.to_string_lossy().into_owned();
    let report = ember(&["inspect", "--cycle", &package_arg], &root);
    assert_eq!(
        report.exit, 0,
        "package-root inspection failed:\n{}",
        report.stderr
    );
    assert!(
        report.stdout.contains("scene.Node.next -> scene.Node"),
        "package-root inspection omitted the imported module's graph:\n{}",
        report.stdout
    );

    std::fs::remove_dir_all(package).expect("temporary package directory is removable");
}

#[test]
fn cycle_explanation_uses_the_same_root_and_graph_as_inspection() {
    let root = workspace_root();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("cycle_inspection.{SOURCE_EXT}"));
    let fixture = fixture
        .strip_prefix(&root)
        .expect("cycle fixture is inside the workspace")
        .to_string_lossy()
        .into_owned();

    let inspected = ember(&["inspect", "--cycle", &fixture], &root);
    assert_eq!(
        inspected.exit, 0,
        "cycle inspection failed:\n{}",
        inspected.stderr
    );
    let explained = ember(&["explain", "--cycle", &fixture, "Root.child"], &root);
    assert_eq!(
        explained.exit, 0,
        "cycle explanation failed:\n{}",
        explained.stderr
    );
    assert!(
        explained.stdout.contains("Root.child -> Child.leaf -> Leaf.root -> Root"),
        "cycle explanation omitted the selected field's ownership path:\n{}",
        explained.stdout
    );
    assert!(
        explained.stdout.contains("strong Root.child: Child -> Child"),
        "cycle explanation omitted the selected field declaration:\n{}",
        explained.stdout
    );
    assert!(
        inspected.stdout.contains("Root.child -> Child.leaf -> Leaf.root -> Root"),
        "inspection and explanation did not share the ownership graph:\ninspection:\n{}",
        inspected.stdout
    );
}

#[test]
fn cycle_explanation_resolves_qualified_and_failing_targets_in_one_root() {
    let root = workspace_root();
    let package = temporary_directory("cycle-target-resolution");
    let source = package.join("src");
    std::fs::create_dir_all(&source).expect("package source directory is creatable");
    std::fs::write(
        package.join(MANIFEST),
        "[package]\nname = \"cycle_targets\"\nkind = \"bin\"\n",
    )
    .expect("package manifest is writable");
    std::fs::write(
        source.join(ember_branding::source_file("main")),
        "import scene\nimport ecs\n\nfn main():\n    return\n",
    )
    .expect("package root source is writable");
    std::fs::write(
        source.join(ember_branding::source_file("scene")),
        "pub class Node:\n    next: Node\n",
    )
    .expect("scene module is writable");
    std::fs::write(
        source.join(ember_branding::source_file("ecs")),
        "pub class Node:\n    next: Node\n",
    )
        .expect("ecs module is writable");

    let package_arg = package.to_string_lossy().into_owned();
    let qualified = ember(
        &["explain", "--cycle", &package_arg, "scene::Node.next"],
        &root,
    );
    assert_eq!(
        qualified.exit, 0,
        "qualified explanation failed:\n{}",
        qualified.stderr
    );
    assert!(
        qualified
            .stdout
            .contains("scene::Node.next -> scene::Node"),
        "qualified explanation did not use Ember qualification:\n{}",
        qualified.stdout
    );

    let missing_class = ember(
        &["explain", "--cycle", &package_arg, "scene::Missing"],
        &root,
    );
    assert_ne!(missing_class.exit, 0, "missing class unexpectedly resolved");
    assert!(
        missing_class.stderr.contains("cannot find class `scene::Missing`"),
        "missing class used the wrong diagnostic:\n{}",
        missing_class.stderr
    );

    let missing_field = ember(
        &["explain", "--cycle", &package_arg, "scene::Node.missing"],
        &root,
    );
    assert_ne!(missing_field.exit, 0, "missing field unexpectedly resolved");
    assert!(
        missing_field.stderr.contains("has no field `missing`"),
        "missing field used the wrong diagnostic:\n{}",
        missing_field.stderr
    );

    let ambiguous = ember(&["explain", "--cycle", &package_arg, "Node"], &root);
    assert_ne!(ambiguous.exit, 0, "ambiguous class unexpectedly resolved");
    assert!(
        ambiguous.stderr.contains("scene::Node") && ambiguous.stderr.contains("ecs::Node"),
        "ambiguity did not name each qualified candidate:\n{}",
        ambiguous.stderr
    );

    let invalid = package.join("not-a-root").to_string_lossy().into_owned();
    let invalid_root = ember(&["explain", "--cycle", &invalid, "Node"], &root);
    assert_ne!(invalid_root.exit, 0, "invalid root unexpectedly resolved");
    assert!(
        invalid_root.stderr.contains("must be an existing package directory"),
        "invalid root did not fail during root resolution:\n{}",
        invalid_root.stderr
    );
    assert!(
        !invalid_root.stderr.contains("scene::Node"),
        "invalid root fell back to the earlier package:\n{}",
        invalid_root.stderr
    );

    std::fs::remove_dir_all(package).expect("temporary package directory is removable");
}

#[test]
fn cycle_explanation_covers_standalone_generic_dynamic_and_stale_roots() {
    let root = workspace_root();
    let generic = format!(
        "tests/conformance/WK-7/warning_generic_class_cycle_after_substitution.{SOURCE_EXT}"
    );
    let class = ember(&["explain", "--cycle", &generic, "Parent"], &root);
    assert_eq!(
        class.exit, 0,
        "class explanation failed:\n{}",
        class.stderr
    );
    assert!(
        class.stdout.contains("Cycle explanation: Parent")
            && class.stdout.contains("Holder[Child]"),
        "class explanation omitted the generic ownership path:\n{}",
        class.stdout
    );
    let field = ember(
        &["explain", "--cycle", &generic, "Parent.holder"],
        &root,
    );
    assert_eq!(
        field.exit, 0,
        "field explanation failed:\n{}",
        field.stderr
    );
    assert!(
        field.stdout.contains("Holder[Child]"),
        "field explanation did not show the instantiated generic owner:\n{}",
        field.stdout
    );

    let directory = temporary_directory("cycle-dynamic-root");
    let dynamic = directory.join(ember_branding::source_file("dynamic"));
    std::fs::write(
        &dynamic,
        "interface Link:\n    fn ping(self)\n\nclass Node:\n    next: Box[dyn Link]\n\nfn main():\n    return\n",
    )
    .expect("dynamic-cycle source is writable");
    let dynamic_arg = dynamic.to_string_lossy().into_owned();
    let dynamic_report = ember(
        &["explain", "--cycle", &dynamic_arg, "Node.next"],
        &root,
    );
    assert_eq!(
        dynamic_report.exit, 0,
        "dynamic explanation failed:\n{}",
        dynamic_report.stderr
    );
    assert!(
        dynamic_report
            .stdout
            .contains("dynamically cycle-capable; no static cycle was proved"),
        "dynamic explanation overclaimed a static cycle:\n{}",
        dynamic_report.stdout
    );

    let unrelated = directory.join(ember_branding::source_file("unrelated"));
    std::fs::write(
        &unrelated,
        "class Unrelated:\n    next: Unrelated\n\nfn main():\n    return\n",
    )
    .expect("unrelated source is writable");
    let unrelated_arg = unrelated.to_string_lossy().into_owned();
    let inspected = ember(&["inspect", "--cycle", &unrelated_arg], &root);
    assert_eq!(
        inspected.exit, 0,
        "unrelated inspection failed:\n{}",
        inspected.stderr
    );
    assert!(
        inspected.stdout.contains("Unrelated.next -> Unrelated"),
        "unrelated inspection did not establish its graph:\n{}",
        inspected.stdout
    );
    let invalid = directory.join("missing-root").to_string_lossy().into_owned();
    let no_fallback = ember(&["explain", "--cycle", &invalid, "Unrelated"], &root);
    assert_ne!(no_fallback.exit, 0, "invalid root unexpectedly resolved");
    assert!(
        !no_fallback.stderr.contains("Unrelated")
            && !no_fallback.stdout.contains("Ownership graph"),
        "invalid root consulted a stale unrelated analysis:\nstderr:\n{}\nstdout:\n{}",
        no_fallback.stderr,
        no_fallback.stdout
    );

    std::fs::remove_dir_all(directory).expect("temporary source directory is removable");
}

#[test]
fn cycle_lint_offers_only_safe_weak_suggestions() {
    let root = workspace_root();
    let direct = ember(
        &[
            "check",
            "--json",
            &format!("tests/conformance/WK-5/warning_direct_strong_class_cycle.{SOURCE_EXT}"),
        ],
        &root,
    );
    assert_eq!(direct.exit, 0, "direct-cycle check failed:\n{}", direct.stderr);
    assert!(
        direct.stdout.contains("\"replacement\":\"Weak[Child]\""),
        "direct class edge lacks a machine-applicable Weak replacement:\n{}",
        direct.stdout
    );

    let shared = ember(
        &[
            "check",
            "--json",
            &format!("tests/conformance/WK-5/warning_shared_class_cycle.{SOURCE_EXT}"),
        ],
        &root,
    );
    assert_eq!(shared.exit, 0, "Shared-cycle check failed:\n{}", shared.stderr);
    assert!(
        shared.stdout.contains("\"suggestions\":[]"),
        "a Shared ownership contract must not receive an automatic Weak rewrite:\n{}",
        shared.stdout
    );
}

#[test]
fn leak_check_is_run_only() {
    let root = workspace_root();
    let checked = ember(
        &[
            "check",
            "--leak-check",
            &format!("tests/conformance/WK-8/accept_runtime_strong_self_cycle.{SOURCE_EXT}"),
        ],
        &root,
    );
    assert_ne!(checked.exit, 0, "`check --leak-check` unexpectedly succeeded");
    assert!(
        checked.stderr.contains("only valid with `ember run`"),
        "run-only leak-check rejection was unclear:\n{}",
        checked.stderr
    );
}

#[test]
fn leak_check_reports_a_live_strong_object_cycle() {
    let root = workspace_root();
    let out_dir = std::env::temp_dir().join(format!(
        "ember-leak-check-{}",
        std::process::id()
    ));
    let out_dir = out_dir.to_string_lossy().into_owned();
    let report = ember(
        &[
            "run",
            "--leak-check",
            &format!("tests/conformance/WK-8/accept_runtime_strong_self_cycle.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir,
        ],
        &root,
    );
    assert_eq!(report.exit, 0, "leak-check run failed:\n{}", report.stderr);
    assert!(
        report.stderr.contains("runtime ownership cycle"),
        "leak-check omitted the runtime SCC report:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("Node.next"),
        "leak-check omitted the owning field:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("strong edges: 1"),
        "leak-check omitted the strong-edge count:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("statically predicted: yes"),
        "leak-check omitted static-cycle correlation:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("suggested weak edge: none"),
        "leak-check offered a Weak rewrite for an Option ownership contract:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("weak Node.previous"),
        "leak-check omitted the weak ownership vocabulary:\n{}",
        report.stderr
    );
}

#[test]
fn leak_check_reports_a_shared_payload_cycle_without_static_overclaim() {
    let root = workspace_root();
    let out_dir = std::env::temp_dir().join(format!(
        "ember-shared-leak-check-{}",
        std::process::id()
    ));
    let out_dir = out_dir.to_string_lossy().into_owned();
    let report = ember(
        &[
            "run",
            "--leak-check",
            &format!("tests/conformance/WK-8/accept_runtime_shared_self_cycle.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir,
        ],
        &root,
    );
    assert_eq!(report.exit, 0, "leak-check run failed:\n{}", report.stderr);
    assert!(
        report.stderr.contains("strong Shared[Node].value"),
        "leak-check omitted the Shared payload edge:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("statically predicted: no"),
        "leak-check claimed the class-only lint predicted a Shared cycle:\n{}",
        report.stderr
    );
}

fn profiles(expectations: &Expectations) -> Vec<&str> {
    if expectations.profiles.is_empty() {
        vec!["debug"]
    } else {
        expectations.profiles.iter().map(String::as_str).collect()
    }
}

/// Assert only against rendered help lines. Searching all of stderr would let
/// a source annotation satisfy itself when the renderer echoes the source,
/// the same false-positive that [`without_source_echo`] prevents for codes.
/// Required help fragments are ordered so policy such as `[DIA-9]` can prove
/// that the structural repair precedes an interior-mutability alternative.
fn assert_diagnostic_helps(relative: &str, stderr: &str, expectations: &Expectations) {
    let help_lines = without_source_echo(stderr)
        .lines()
        .filter(|line| line.trim_start().starts_with("= help:"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut after = 0;
    for required in &expectations.helps {
        let Some(found) = help_lines[after..].find(required) else {
            panic!(
                "{relative}: expected ordered help containing {required:?}\n--- help lines ---\n{help_lines}\n--- stderr ---\n{stderr}"
            );
        };
        after += found + required.len();
    }
    for forbidden in &expectations.forbidden_helps {
        assert!(
            !help_lines.contains(forbidden),
            "{relative}: diagnostic help must not contain {forbidden:?}\n--- help lines ---\n{help_lines}\n--- stderr ---\n{stderr}"
        );
    }
}

fn check_file(path: &Path, root: &Path) {
    let source = std::fs::read_to_string(path).expect("the test file is readable");
    let expectations = parse_expectations(&source);
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();

    // `parse-pass` / `parse-fail` run `ember check --syntax-only` (`[CLI-9]`),
    // which reports only `E00xx` and `E01xx`. They are how a rule whose
    // *grammar* has landed ahead of its semantics gets a real `[TST-4a]`
    // accept-and-reject pair instead of a directory holding an aspiration.
    // The file may name types the compiler cannot yet resolve, because
    // `--syntax-only` does not resolve names.
    match expectations.kind.as_deref() {
        Some("parse-pass") => {
            let checked = ember(&["check", "--syntax-only", &relative], root);
            assert_eq!(
                checked.exit, 0,
                "{relative}: expected it to parse\nstderr:\n{}",
                checked.stderr
            );
            return;
        }
        Some("parse-fail") => {
            let checked = ember(&["check", "--syntax-only", &relative], root);
            assert_ne!(
                checked.exit, 0,
                "{relative}: expected a parse error, but it parsed"
            );
            for (code, message) in &expectations.errors {
                assert!(
                    checked.stderr.contains(code.as_str()),
                    "{relative}: expected {code}\nstderr:\n{}",
                    checked.stderr
                );
                assert!(
                    checked.stderr.contains(message.as_str()),
                    "{relative}: expected a message containing {message:?}\nstderr:\n{}",
                    checked.stderr
                );
            }
            return;
        }
        _ => {}
    }
    let out_dir = std::env::temp_dir().join("ember-tests").join(
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
    );
    let profiles = profiles(&expectations);

    // A `compile-fail` test must be rejected, with the diagnostics it names.
    if !expectations.errors.is_empty() || expectations.kind.as_deref() == Some("compile-fail") {
        for profile in &profiles {
            let checked = ember(&["check", &relative, "--profile", profile], root);
            assert_ne!(
                checked.exit, 0,
                "{relative} [{profile}]: expected compilation to fail, but it succeeded"
            );
            let said = without_source_echo(&checked.stderr);
            for (code, message) in &expectations.errors {
                assert!(
                    said.contains(code.as_str()),
                    "{relative} [{profile}]: expected {code}
stderr:
{}",
                    checked.stderr
                );
                assert!(
                    said.contains(message.as_str()),
                    "{relative} [{profile}]: expected a message containing {message:?}
stderr:
{}",
                    checked.stderr
                );
            }
            assert_diagnostic_helps(&relative, &checked.stderr, &expectations);
        }
        return;
    }

    // `[TST-1]` — warnings and lints the file names must be produced.
    if !expectations.warnings.is_empty() {
        for profile in &profiles {
            let checked = ember(&["check", &relative, "--profile", profile], root);
            let said = without_source_echo(&checked.stderr);
            for (code, message) in &expectations.warnings {
                assert!(
                    said.contains(code.as_str()),
                    "{relative} [{profile}]: expected {code}
stderr:
{}",
                    checked.stderr
                );
                assert!(
                    said.contains(message.as_str()),
                    "{relative} [{profile}]: expected a warning containing {message:?}
stderr:
{}",
                    checked.stderr
                );
            }
            assert_diagnostic_helps(&relative, &checked.stderr, &expectations);
        }
    }

    // The emitted C, for `assert-c-order`.
    if !expectations.assert_c_order.is_empty() {
        for profile in &profiles {
            let emitted = ember(
                &["build", &relative, "--emit", "c", "--profile", profile],
                root,
            );
            assert_eq!(
                emitted.exit, 0,
                "emitting C for {relative} [{profile}] failed:\n{}",
                emitted.stderr
            );
            for (first, second) in &expectations.assert_c_order {
                let at_first = emitted.stdout.find(first.as_str());
                let at_second = emitted.stdout.find(second.as_str());
                let (Some(at_first), Some(at_second)) = (at_first, at_second) else {
                    panic!(
                        "{relative} [{profile}]: assert-c-order needs both needles present; {} is missing\n--- emitted C ---\n{}",
                        if at_first.is_none() {
                            format!("{first:?}")
                        } else {
                            format!("{second:?}")
                        },
                        emitted.stdout
                    );
                };
                assert!(
                    at_first < at_second,
                    "{relative} [{profile}]: expected {first:?} before {second:?}, found it after\n--- emitted C ---\n{}",
                    emitted.stdout
                );
            }
        }
    }

    // The emitted C, for `assert-c`.
    if !expectations.assert_c.is_empty() {
        for profile in &profiles {
            let emitted = ember(
                &["build", &relative, "--emit", "c", "--profile", profile],
                root,
            );
            assert_eq!(
                emitted.exit, 0,
                "emitting C for {relative} [{profile}] failed:\n{}",
                emitted.stderr
            );
            for (expect_present, needle) in &expectations.assert_c {
                let present = emitted.stdout.contains(needle.as_str());
                assert_eq!(
                    present,
                    *expect_present,
                    "{relative} [{profile}]: expected the emitted C {} {needle:?}\n--- emitted C ---\n{}",
                    if *expect_present {
                        "to contain"
                    } else {
                        "not to contain"
                    },
                    emitted.stdout
                );
            }
        }
    }

    // The emitted C, for exact occurrence-count assertions. This is stronger
    // than a presence check for rules such as `[ARN-5]`, where construction
    // allocates once and every subsequent operation must reuse that storage.
    if !expectations.assert_c_count.is_empty() {
        for profile in &profiles {
            let emitted = ember(
                &["build", &relative, "--emit", "c", "--profile", profile],
                root,
            );
            assert_eq!(
                emitted.exit, 0,
                "emitting C for {relative} [{profile}] failed:\n{}",
                emitted.stderr
            );
            for (needle, expected) in &expectations.assert_c_count {
                let actual = emitted.stdout.matches(needle.as_str()).count();
                assert_eq!(
                    actual, *expected,
                    "{relative} [{profile}]: expected the emitted C to contain {needle:?} exactly {expected} time(s), found {actual}\n--- emitted C ---\n{}",
                    emitted.stdout
                );
            }
        }
    }

    for profile in &profiles {
        let profile_out_dir = out_dir.join(profile);
        let out_dir_arg = profile_out_dir.to_string_lossy().into_owned();
        let run = ember(
            &[
                "run",
                &relative,
                "--out-dir",
                &out_dir_arg,
                "--profile",
                profile,
            ],
            root,
        );

        // A `run-fail` test compiles and runs, then panics with a given message.
        if let Some(message) = &expectations.panics {
            assert_ne!(
                run.exit, 0,
                "{relative} [{profile}]: expected a panic, but the program exited cleanly
stdout:
{}",
                run.stdout
            );
            assert!(
                run.stderr.contains(message.as_str()),
                "{relative} [{profile}]: expected a panic mentioning {message:?}
stderr:
{}",
                run.stderr
            );
            // A panic contract may also require proving that execution did
            // not reach a continuation marker. `stdout` used to be ignored
            // for every run-fail case, so such a fixture could only assert
            // that *some* panic happened, not where execution stopped.
            if let Some(expected) = &expectations.stdout {
                assert_eq!(
                    run.stdout.trim_end(),
                    expected.trim_end(),
                    "{relative} [{profile}]: stdout before panic differs\nstderr:\n{}",
                    run.stderr
                );
            }
            continue;
        }

        if let Some(expected) = &expectations.stdout {
            assert_eq!(
                run.stdout.trim_end(),
                expected.trim_end(),
                "{relative} [{profile}]: stdout differs\nstderr:\n{}",
                run.stderr
            );
        }
        let expected_exit = expectations.exit.unwrap_or(0);
        assert_eq!(
            run.exit, expected_exit,
            "{relative} [{profile}]: exit code differs\nstdout:\n{}\nstderr:\n{}",
            run.stdout, run.stderr
        );
    }
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
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e == ember_branding::SOURCE_EXT)
        })
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
    let count = check_directory("tests/run-pass");
    assert!(count > 0, "no run-pass programs were found");
}

/// `tests/compile-pass` was walked by nothing until 2026-09-08: the directory
/// existed, held files, and no test read them. Every suite asserts it found
/// something for the same reason.
#[test]
fn compile_pass_programs_compile() {
    let count = check_directory("tests/compile-pass");
    assert!(count > 0, "no compile-pass programs were found");
}

#[test]
fn compile_fail_programs_are_rejected() {
    let count = check_directory("tests/compile-fail");
    assert!(count > 0, "no compile-fail programs were found");
}

#[test]
fn run_fail_programs_panic() {
    let count = check_directory("tests/run-fail");
    assert!(count > 0, "no run-fail programs were found");
}

/// `[FMT-1]` — `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡ parse(x)`, over
/// the whole corpus, which is what the rule asks for.
#[test]
fn formatting_is_idempotent_and_preserves_the_tree() {
    let root = workspace_root();
    let mut checked = 0;
    for directory in ["tests/run-pass", "tests/milestones", "examples"] {
        let dir = root.join(directory);
        if !dir.is_dir() {
            continue;
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
            .expect("the directory is readable")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .is_some_and(|e| e == ember_branding::SOURCE_EXT)
            })
            .collect();
        entries.sort();
        for path in entries {
            let relative = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            let once = ember(&["fmt", &relative], &root);
            assert_eq!(
                once.exit, 0,
                "`ember fmt {relative}` failed:\n{}",
                once.stderr
            );

            // Formatting the output again must change nothing.
            let scratch = std::env::temp_dir().join("ember-fmt-once.em");
            std::fs::write(&scratch, &once.stdout).expect("the scratch file is writable");
            let twice = ember(&["fmt", &scratch.to_string_lossy()], &root);
            assert_eq!(
                once.stdout, twice.stdout,
                "{relative}: fmt(fmt(x)) differs from fmt(x)"
            );

            // And the formatted source must parse to the same tree.
            let before = ember(&["build", &relative, "--emit", "ast"], &root);
            let after = ember(
                &["build", &scratch.to_string_lossy(), "--emit", "ast"],
                &root,
            );
            assert_eq!(
                before.stdout, after.stdout,
                "{relative}: parse(fmt(x)) differs from parse(x)"
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "no files were formatted");
}

#[test]
fn hello_world_builds_and_runs() {
    // Phase 0's exit criterion (Part XX.2).
    let root = workspace_root();
    let out_dir = std::env::temp_dir().join("ember-tests").join("hello");
    let hello = format!("examples/{}", ember_branding::source_file("hello"));
    let run = ember(
        &[
            "run",
            &hello,
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        &root,
    );
    assert_eq!(run.exit, 0, "hello.em failed:\n{}", run.stderr);
    assert_eq!(run.stdout.trim_end(), "hello, world");
}

/// `[LT-40]` / `[BLD-2]` — changing an imported callable's verified region
/// summary changes the imported module's interface hash and therefore the
/// caller's cache key, even though the caller source stays byte-identical.
/// This proves a real cross-build dependency identity rather than merely an
/// in-memory metadata fingerprint.
#[test]
fn callable_region_summary_changes_invalidate_importers_interface_key() {
    let workspace = workspace_root();
    let test_root =
        std::env::temp_dir().join(format!("ember-lt40-interface-{}", std::process::id()));
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create LT-40 package");
    std::fs::write(
        test_root.join(&helper),
        "pub fn select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n\npub(package) fn package_select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n\npub fn signature_only(value: i32) -> i32:\n    return value\n\npub unsafe extern \"C\" fn abi_only(value: i32) -> i32:\n    return value\n\nfn hidden(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n",
    )
    .expect("write initial helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import select, package_select\n\nfn main():\n    first: Array[i32] = Array[i32]()\n    first.push(10)\n    second: Array[i32] = Array[i32]()\n    second.push(20)\n    selected = select(first.as_span(), second.as_span())\n    package_selected = package_select(first.as_span(), second.as_span())\n    println(selected[0])\n    println(package_selected[0])\n",
    )
    .expect("write importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for LT-40");
        assert!(
            output.status.success(),
            "{label} check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check("initial");
    let before_root = cached_interface(&out_dir, "root");
    let before_helper = cached_interface(&out_dir, "helper");
    assert_eq!(before_helper.callables.len(), 4);
    assert!(before_helper
        .callables
        .contains_key(&ember_branding::mangled("helper.select")));
    assert!(before_helper
        .callables
        .contains_key(&ember_branding::mangled("helper.package_select")));
    let signature = &before_helper.callables[&ember_branding::mangled("helper.signature_only")]
        .signature;
    assert_eq!(signature.parameters.len(), 1);
    assert_eq!(signature.parameters[0].ty, "i32");
    assert_eq!(signature.result, "i32");
    let abi = &before_helper.callables["abi_only"].signature;
    assert!(abi.is_unsafe);
    assert_eq!(abi.abi.as_deref(), Some("C"));

    std::fs::write(
        test_root.join(&helper),
        "pub fn select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n\npub(package) fn package_select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n\npub fn signature_only(value: i32) -> i32:\n    return value\n\npub unsafe extern \"C\" fn abi_only(value: i32) -> i32:\n    return value\n\nfn hidden(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n",
    )
    .expect("change private callable summary");
    check("after private summary change");
    let after_private_root = cached_interface(&out_dir, "root");
    let after_private_helper = cached_interface(&out_dir, "helper");

    assert_ne!(before_helper.cache_key, after_private_helper.cache_key);
    assert_eq!(before_helper.interface_hash, after_private_helper.interface_hash);
    assert_eq!(before_root.cache_key, after_private_root.cache_key);

    std::fs::write(
        test_root.join(&helper),
        "pub fn select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return second\n\npub(package) fn package_select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n\npub fn signature_only(value: i32) -> i32:\n    return value\n\npub unsafe extern \"C\" fn abi_only(value: i32) -> i32:\n    return value\n\nfn hidden(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n",
    )
    .expect("change package-visible callable summary");
    check("after package-visible summary change");
    let after_package_root = cached_interface(&out_dir, "root");
    let after_package_helper = cached_interface(&out_dir, "helper");

    assert_ne!(after_private_helper.interface_hash, after_package_helper.interface_hash);
    assert_ne!(after_private_root.cache_key, after_package_root.cache_key);

    std::fs::write(
        test_root.join(&helper),
        "pub fn select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n\npub(package) fn package_select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n\npub fn signature_only(value: i32) -> i32:\n    return value\n\npub unsafe extern \"C\" fn abi_only(value: i32) -> i32:\n    return value\n\nfn hidden(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n",
    )
    .expect("change public callable summary");
    check("after public summary change");
    let after_public_root = cached_interface(&out_dir, "root");
    let after_public_helper = cached_interface(&out_dir, "helper");

    assert_ne!(after_package_helper.interface_hash, after_public_helper.interface_hash);
    assert_ne!(after_package_root.cache_key, after_public_root.cache_key);

    std::fs::write(
        test_root.join(&helper),
        "pub fn select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n\npub(package) fn package_select(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n\npub fn signature_only(value: i64) -> i64:\n    return value\n\npub unsafe extern \"C\" fn abi_only(value: i32) -> i32:\n    return value\n\nfn hidden(first: Span[i32], second: Span[i32]) -> Span[i32]:\n    return first\n",
    )
    .expect("change public callable signature");
    check("after public signature change");
    let after_signature_root = cached_interface(&out_dir, "root");
    let after_signature_helper = cached_interface(&out_dir, "helper");

    assert_ne!(after_public_helper.interface_hash, after_signature_helper.interface_hash);
    assert_ne!(after_public_root.cache_key, after_signature_root.cache_key);
    let signature = &after_signature_helper.callables
        [&ember_branding::mangled("helper.signature_only")]
        .signature;
    assert_eq!(signature.parameters[0].ty, "i64");
    assert_eq!(signature.result, "i64");
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[FN-1]` / `[FFI-5]` / `[BLD-2]` — the cache records declaration modes,
/// rather than MIR's implementation `ref mut` spelling, and retains the ABI
/// and unsafe call boundary as caller-visible signature facts.
#[test]
fn callable_signature_modes_unsafe_and_abi_cross_the_interface_boundary() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-callable-signature-{}",
        std::process::id()
    ));
    let out_dir = test_root.join("target");
    let source = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create callable-signature package");
    std::fs::write(
        test_root.join(&source),
        "pub fn replace(mut value: i32, owned replacement: i32) -> i32:\n    value = replacement\n    return value\n\npub unsafe extern \"C\" fn native_boundary(value: i32) -> i32:\n    return value\n\nfn main():\n    println(1)\n",
    )
    .expect("write callable-signature source");

    let output = Command::new(EMBER)
        .args(["check", &source, "--out-dir", &out_dir.to_string_lossy()])
        .current_dir(&test_root)
        .env(ember_branding::std_path_var(), workspace.join("std"))
        .output()
        .expect("the Ember compiler runs for callable signatures");
    assert!(
        output.status.success(),
        "callable-signature check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let artifact = cached_interface(&out_dir, "root");
    let replace = &artifact.callables[&ember_branding::mangled("replace")].signature;
    assert_eq!(replace.parameters.len(), 2);
    assert_eq!(replace.parameters[0].mode, CallableParameterMode::Mut);
    assert_eq!(replace.parameters[0].ty, "i32");
    assert_eq!(replace.parameters[1].mode, CallableParameterMode::Owned);
    assert_eq!(replace.parameters[1].ty, "i32");
    assert_eq!(replace.result, "i32");
    assert!(!replace.is_unsafe);
    assert_eq!(replace.abi, None);

    let native = &artifact.callables["native_boundary"].signature;
    assert_eq!(native.parameters[0].mode, CallableParameterMode::Borrow);
    assert!(native.is_unsafe);
    assert_eq!(native.abi.as_deref(), Some("C"));
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[TYP-16]` / `[TYP-17]` / `[BLD-2]` — a public generic declaration is an
/// interface even before (or independently of) a monomorphized MIR body. A
/// bound change must invalidate importers without treating one specialization
/// as the declaration's whole contract.
#[test]
fn generic_callable_bounds_invalidate_importers_without_an_emitted_declaration_body() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-generic-interface-{}",
        std::process::id()
    ));
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create generic-interface package");
    std::fs::write(
        test_root.join(&helper),
        "pub fn identity[T: Eq](value: T) -> T:\n    return value\n",
    )
    .expect("write initial generic helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import identity\n\nfn main():\n    println(identity(1))\n",
    )
    .expect("write generic importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for generic interfaces");
        assert!(
            output.status.success(),
            "{label} generic-interface check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check("initial");
    let before_helper = cached_interface(&out_dir, "helper");
    let before_root = cached_interface(&out_dir, "root");
    let identity = &before_helper.callables[&ember_branding::mangled("helper.identity")];
    assert!(identity.metadata.is_none());
    assert_eq!(identity.signature.generics.len(), 1);
    assert_eq!(identity.signature.generics[0].bounds.len(), 1);

    std::fs::write(
        test_root.join(&helper),
        "pub fn identity[T: Hash](value: T) -> T:\n    return value\n",
    )
    .expect("change generic bound");
    check("after generic bound change");
    let after_helper = cached_interface(&out_dir, "helper");
    let after_root = cached_interface(&out_dir, "root");

    assert_ne!(before_helper.interface_hash, after_helper.interface_hash);
    assert_ne!(before_root.cache_key, after_root.cache_key);
    let identity = &after_helper.callables[&ember_branding::mangled("helper.identity")];
    assert!(identity.metadata.is_none());
    assert_eq!(identity.signature.generics.len(), 1);
    assert_eq!(identity.signature.generics[0].bounds.len(), 1);
    assert_ne!(
        before_helper.callables[&ember_branding::mangled("helper.identity")]
            .signature
            .generics[0]
            .bounds,
        identity.signature.generics[0].bounds
    );
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[FN-6a]` / `[BLD-2]` — modes inside an implicit `Callable` bound are
/// source-level type identity.  An import cannot be reused after `fn(mut T)`
/// becomes `fn(T)`, even though both source spellings contain the same `T`.
#[test]
fn generic_callable_parameter_modes_cross_the_interface_boundary() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-callable-mode-interface-{}",
        std::process::id()
    ));
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create callable-mode package");
    std::fs::write(
        test_root.join(&helper),
        "pub fn apply(f: fn(mut i32) -> i32, mut value: i32) -> i32:\n    return f(value)\n",
    )
    .expect("write initial callable-mode helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import apply\n\nfn increment(mut value: i32) -> i32:\n    value = value + 1\n    return value\n\nfn main():\n    value = 4\n    println(apply(increment, value))\n",
    )
    .expect("write callable-mode importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for callable-mode interfaces");
        assert!(
            output.status.success(),
            "{label} callable-mode interface check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check("initial");
    let before_helper = cached_interface(&out_dir, "helper");
    let before_root = cached_interface(&out_dir, "root");
    let apply = &before_helper.callables[&ember_branding::mangled("helper.apply")];
    let bound = apply.signature.generics[0]
        .callable
        .as_ref()
        .expect("fn parameter has an implicit Callable bound");
    assert_eq!(bound.parameters.len(), 1);
    assert_eq!(bound.parameters[0].mode, CallableParameterMode::Mut);
    assert_eq!(bound.parameters[0].ty, "i32");

    std::fs::write(
        test_root.join(&helper),
        "pub fn apply(f: fn(i32) -> i32, mut value: i32) -> i32:\n    return f(value)\n",
    )
    .expect("change callable-mode helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import apply\n\nfn identity(value: i32) -> i32:\n    return value\n\nfn main():\n    value = 4\n    println(apply(identity, value))\n",
    )
    .expect("adapt importer to the changed callable contract");
    check("after callable mode change");
    let after_helper = cached_interface(&out_dir, "helper");
    let after_root = cached_interface(&out_dir, "root");
    assert_ne!(before_helper.interface_hash, after_helper.interface_hash);
    assert_ne!(before_root.cache_key, after_root.cache_key);
    let bound = after_helper.callables[&ember_branding::mangled("helper.apply")]
        .signature
        .generics[0]
        .callable
        .as_ref()
        .expect("changed fn parameter remains an implicit Callable bound");
    assert_eq!(bound.parameters[0].mode, CallableParameterMode::Borrow);
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[TYP-16]` / `[TYP-17]` / `[IFC-1]` / `[MOD-2]` / `[BLD-2]` — members are
/// source declarations too. In particular, a generic owner's receiver has no
/// concrete runtime type until an instantiation exists, so an emitted
/// specialization must not be used as the public declaration contract.
#[test]
fn visible_member_declarations_preserve_generic_owner_interface_identity() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-member-interface-{}",
        std::process::id()
    ));
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create member-interface package");
    std::fs::write(
        test_root.join(&helper),
        "from std.core import Eq\nfrom std.collections import Hash\n\npub struct Holder[T: Eq]:\n    value: T\n\n    pub fn mirror[U: Eq](self, value: U) -> U:\n        return value\n\npub struct Meter:\n    value: i32\n\nextend Meter:\n    pub fn current(self) -> i32:\n        return self.value\n\n    fn hidden(self) -> i32:\n        return self.value\n\npub interface Identity:\n    pub fn keep[U: Eq](self, value: U) -> U\n",
    )
    .expect("write initial member helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import Holder, Meter, Identity\n\nfn main():\n    println(1)\n",
    )
    .expect("write member importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for member interfaces");
        assert!(
            output.status.success(),
            "{label} member-interface check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check("initial");
    let before_helper = cached_interface(&out_dir, "helper");
    let before_root = cached_interface(&out_dir, "root");

    let holder = before_helper
        .callables
        .values()
        .find(|contract| {
            contract.metadata.is_none()
                && contract.signature.parameters.first().is_some_and(|parameter| {
                    parameter.ty == "helper.Holder[$P0]"
                        && parameter.mode == CallableParameterMode::Borrow
                })
    })
        .expect("generic-owner member declaration is serialized");
    assert_eq!(holder.signature.generics.len(), 2);
    let initial_owner_bounds = holder.signature.generics[0].bounds.clone();
    assert_eq!(initial_owner_bounds, holder.signature.generics[1].bounds);

    let extension = before_helper
        .callables
        .values()
        .find(|contract| {
            contract.metadata.is_some()
                && contract.signature.parameters.first().is_some_and(|parameter| {
                    parameter.ty == "helper.Meter"
                        && parameter.mode == CallableParameterMode::Borrow
                })
        })
        .expect("extension member declaration receives its verified body summary");
    assert_eq!(extension.signature.result, "i32");

    let interface = before_helper
        .callables
        .values()
        .find(|contract| {
            contract.metadata.is_none()
                && contract.signature.parameters.first().is_some_and(|parameter| {
                    parameter.ty == "interface:helper.Identity::Self"
                        && parameter.mode == CallableParameterMode::Borrow
                })
    })
        .expect("interface member declaration is serialized without a fabricated body");
    assert_eq!(interface.signature.generics.len(), 1);
    assert_eq!(interface.signature.generics[0].bounds, initial_owner_bounds);

    std::fs::write(
        test_root.join(&helper),
        "from std.core import Eq\nfrom std.collections import Hash\n\npub struct Holder[T: Eq]:\n    value: T\n\n    pub fn mirror[U: Eq](self, value: U) -> U:\n        return value\n\npub struct Meter:\n    value: i32\n\nextend Meter:\n    pub fn current(self) -> i32:\n        return self.value\n\n    fn hidden(self) -> i32:\n        return 0\n\npub interface Identity:\n    pub fn keep[U: Eq](self, value: U) -> U\n",
    )
    .expect("change private member body");
    check("after private member body change");
    let after_private_helper = cached_interface(&out_dir, "helper");
    let after_private_root = cached_interface(&out_dir, "root");
    assert_eq!(before_helper.interface_hash, after_private_helper.interface_hash);
    assert_eq!(before_root.cache_key, after_private_root.cache_key);

    std::fs::write(
        test_root.join(&helper),
        "from std.core import Eq\nfrom std.collections import Hash\n\npub struct Holder[T: Hash]:\n    value: T\n\n    pub fn mirror[U: Eq](self, value: U) -> U:\n        return value\n\npub struct Meter:\n    value: i32\n\nextend Meter:\n    pub fn current(self) -> i32:\n        return self.value\n\n    fn hidden(self) -> i32:\n        return 0\n\npub interface Identity:\n    pub fn keep[U: Eq](self, value: U) -> U\n",
    )
    .expect("change generic owner bound");
    check("after generic owner bound change");
    let after_helper = cached_interface(&out_dir, "helper");
    let after_root = cached_interface(&out_dir, "root");

    assert_ne!(after_private_helper.interface_hash, after_helper.interface_hash);
    assert_ne!(after_private_root.cache_key, after_root.cache_key);
    let holder = after_helper
        .callables
        .values()
        .find(|contract| {
            contract.signature.parameters.first().is_some_and(|parameter| {
                parameter.ty == "helper.Holder[$P0]"
                    && parameter.mode == CallableParameterMode::Borrow
            })
        })
        .expect("changed generic-owner declaration remains serialized");
    assert_ne!(initial_owner_bounds, holder.signature.generics[0].bounds);
    let _ = std::fs::remove_dir_all(&test_root);
}

fn cached_interface(out_dir: &Path, module: &str) -> ModuleInterfaceArtifact {
    let directory = out_dir.join("debug").join("interface");
    for package in std::fs::read_dir(&directory).expect("interface cache has a package directory") {
        let package = package.expect("read interface package directory").path();
        if !package.is_dir() {
            continue;
        }
        for artifact in std::fs::read_dir(&package).expect("read interface cache files") {
            let artifact = artifact.expect("read interface artifact").path();
            if artifact
                .extension()
                .is_none_or(|extension| extension != "emif")
            {
                continue;
            }
            let bytes = std::fs::read(&artifact).expect("read interface artifact bytes");
            let parsed = ModuleInterfaceArtifact::from_bytes(&bytes)
                .unwrap_or_else(|error| panic!("{}: {error}", artifact.display()));
            if parsed.module == module {
                return parsed;
            }
        }
    }
    panic!("interface artifact for module `{module}` was not written")
}

#[test]
fn the_emitted_c_compiles_without_warnings() {
    // `[CG-C-1]` — warning-free under -std=c11 -Wall -Wextra.
    let root = workspace_root();
    let milestone = format!(
        "tests/milestones/{}",
        ember_branding::source_file("m1_value_code_has_no_runtime_cost")
    );
    let emitted = ember(
        &[
            "build",
            &milestone,
            "--emit",
            "c",
        ],
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
    let alloc = ember_branding::runtime("alloc");
    let arena_alloc = ember_branding::runtime("arena_alloc");
    let source = format!(
        "\
#$ test: run-pass
#$ profiles: debug, release, shipping
struct S:
    x: i32
#$ stdout: 5
#$ assert-c: !contains(\"{alloc}\")
#$ assert-c-count: contains(\"{arena_alloc}\") == 1
#$ help: keep one owner
#$ not-help: RefCell
#$ exit: 0
"
    );
    let parsed = parse_expectations(&source);
    assert_eq!(parsed.kind.as_deref(), Some("run-pass"));
    assert_eq!(parsed.stdout.as_deref(), Some("5"));
    assert_eq!(parsed.exit, Some(0));
    assert_eq!(parsed.profiles, ["debug", "release", "shipping"]);
    assert_eq!(parsed.assert_c, vec![(false, alloc)]);
    assert_eq!(parsed.assert_c_count, vec![(arena_alloc, 1)]);
    assert_eq!(parsed.helps, ["keep one owner"]);
    assert_eq!(parsed.forbidden_helps, ["RefCell"]);
}

/// `[TST-4]`/`[TST-4a]` — every rule directory under `tests/conformance/` is
/// run, and every file in one carries its expectations. A directory that
/// exists but is never executed is what `[TST-4a]` calls "not coverage".
#[test]
fn the_conformance_suite_runs() {
    let root = workspace_root();
    let dir = root.join("tests").join("conformance");
    if !dir.is_dir() {
        return;
    }
    let mut rules: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/conformance is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    rules.sort();
    assert!(
        !rules.is_empty(),
        "tests/conformance holds no rule directories"
    );

    for rule_dir in rules {
        let rule = rule_dir.file_name().unwrap().to_string_lossy().into_owned();
        let mut files: Vec<PathBuf> = std::fs::read_dir(&rule_dir)
            .expect("a rule directory is readable")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .is_some_and(|e| e == ember_branding::SOURCE_EXT)
            })
            .collect();
        files.sort();
        assert!(
            !files.is_empty(),
            "tests/conformance/{rule}/ holds no programs"
        );

        // `[TST-4a]` — an accept case always, and a reject case for a rule
        // that can reject source. Which rules need one is `[TST-4b]`'s
        // mechanical question, answered from the rule→code map; here the
        // file name carries the answer, and a directory with neither is a
        // directory that tests nothing.
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(
            names.iter().any(|n| n.starts_with("accept_")),
            "tests/conformance/{rule}/ has no accept_* case ([TST-4a])"
        );

        for path in &files {
            let source = std::fs::read_to_string(path).expect("readable");
            assert!(
                source.contains("#$ rules:"),
                "{}: no `#$ rules:` annotation",
                path.display()
            );
            check_file(path, &root);
        }
    }
}

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
//! A run test may feed its program standard input, one `#$ stdin: text` line
//! per input line (`[STD-10]`'s `input`); without one, standard input is closed.
//! It may pass its program arguments, one `#$ args: text` line per argument
//! (`[FN-8]`'s `args`).
//!
//! `#$` is an ordinary comment to the compiler — `$` has no other meaning
//! anywhere in Ember — so an annotation can never be mistaken for source and
//! can never affect compilation.
//!
//! `[TST-0]` — annotations are read from the raw source text, not from the
//! token stream: a `compile-fail` test may be expected to fail at the lexer,
//! so its expectations must be readable even when the file does not tokenise.

use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, PoisonError};

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::{TemporaryDirectory, temporary_directory};

use ember_build::interface::{CallableParameterMode, ModuleInterfaceArtifact};
use ember_build::{LinkRequest, Profile, Toolchain};
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
    /// `#$ c-compilers: msvc, clang` — the C compilers this test's C is for; empty means every one.
    c_compilers: Vec<String>,
    stdout: Option<String>,
    /// `#$ stdin:` — one line of standard input per annotation.
    stdin: Option<String>,
    /// `#$ args:` — one program argument per annotation, after `--`.
    args: Vec<String>,
    /// `#$ stderr:` — text the run's standard error must contain, one needle
    /// per annotation: the debug leak report (`[WK-15]`) goes there.
    stderr: Vec<String>,
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
    errors: Vec<(String, String)>,
    /// `[TST-1]` — for each of `errors`, the line the annotation trails,
    /// which its diagnostic's primary span must start on; `None` for an
    /// annotation on a line of its own, which may match anywhere.
    error_lines: Vec<Option<usize>>,
    /// Ordered `= help:` substrings required from a diagnostic.
    helps: Vec<String>,
    /// Text that must not occur in any `= help:` line. This pins negative
    /// suggestion policy such as `[CELL-10]` without matching source comments.
    forbidden_helps: Vec<String>,
    /// `#$ warning[WXXXX]: text` / `#$ warning[LXXXX]: text` — a warning or
    /// lint the compiler must produce. These were parsed by nobody until
    /// 2026-09-08, so a file could assert any warning at all and pass.
    warnings: Vec<(String, String)>,
    /// As `error_lines`, for `warnings`.
    warning_lines: Vec<Option<usize>>,
}

/// The text after `error[` or `warning[`: `E3062]: message`, or `E3062]`
/// alone, which requires the code and no particular message (D-184).
fn code_and_message(value: &str) -> Option<(String, String)> {
    let (code, rest) = value.split_once(']')?;
    let message = rest.strip_prefix(':').unwrap_or(rest);
    Some((code.trim().to_string(), message.trim().to_string()))
}

fn parse_expectations(source: &str) -> Expectations {
    let mut expectations = Expectations::default();
    let mut collecting_stdout = false;
    let mut names_c_compilers = false;
    let mut stdin_lines: Vec<String> = Vec::new();
    let mut stdout_lines: Vec<String> = Vec::new();

    for (index, line) in source.lines().enumerate() {
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
        let (rest, trailing) = match trimmed.strip_prefix("#$") {
            Some(rest) => (rest, None),
            None => match line.find("#$") {
                Some(at) if line[..at].ends_with(char::is_whitespace) => (&line[at + 2..], Some(index + 1)),
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
                    matches!(profile.as_str(), "debug" | "development" | "release"),
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
            // D-184 — `#$ error[E3062]` with no `:` names a code and no message.
            // It used to be dropped, so eight cases passed on any rejection.
            if let Some((code, message)) = code_and_message(value) {
                expectations.errors.push((code, message));
                expectations.error_lines.push(trailing);
            }
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("warning[") {
            if let Some((code, message)) = code_and_message(value) {
                expectations.warnings.push((code, message));
                expectations.warning_lines.push(trailing);
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
        } else if let Some(value) = rest.strip_prefix("args:") {
            expectations.args.push(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("stdin:") {
            stdin_lines.push(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("stderr:") {
            expectations.stderr.push(value.trim().to_string());
            collecting_stdout = false;
        } else if let Some(value) = rest.strip_prefix("stdout:") {
            collecting_stdout = true;
            let value = value.trim();
            if !value.is_empty() {
                stdout_lines.push(value.to_string());
            }
        } else if collecting_stdout {
            stdout_lines.push(rest.to_string());
        } else if let Some(value) = rest.strip_prefix("c-compilers:") {
            // The C compilers this test's C is for (`extern "vectorcall"`, `[FFI-9]`): with any
            // other, the suite and the C warning sweep skip the test.
            expectations.c_compilers = value.split(',').map(str::trim).filter(|name| !name.is_empty()).map(str::to_string).collect();
            names_c_compilers = true;
            collecting_stdout = false;
        } else if !rest.is_empty() && !rest.starts_with("rules:") && !rest.starts_with("note") {
            // An unrecognised key used to fall through in silence, so
            // `#$ panic:` for `#$ panics:` asserted nothing and the test
            // passed. Every annotation is now either understood or refused.
            panic!(
                "unrecognised `#$` annotation: {rest:?}
  known keys:                  test, profiles, exit, assert-c, error[…], warning[…], help, not-help, panics, stdout, stderr, stdin, args, rules, note, c-compilers"
            );
        }
    }

    if !stdout_lines.is_empty() {
        expectations.stdout = Some(stdout_lines.join("\n"));
    }
    assert!(
        !names_c_compilers || matches!(expectations.kind.as_deref(), Some("compile-pass" | "compile-fail")),
        "`#$ c-compilers:` is for a `compile-pass` or `compile-fail` test"
    );
    if !stdin_lines.is_empty() {
        expectations.stdin = Some(stdin_lines.iter().map(|line| format!("{line}\n")).collect());
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

/// `[TST-1]` — "`#$ error[E…]: text`, `#$ warning[…]` and `#$ note` assert a
/// diagnostic whose primary span starts on that line; unexpected and missing
/// diagnostics both fail." Each annotation claims one diagnostic with its code
/// whose text (message, labels, notes or helps) contains the annotation's; one
/// that trails a line of code also needs the diagnostic's primary span to
/// start on that line. A diagnostic no annotation claims fails the test, as a
/// missing one does. The diagnostics are read from `--json`, so a source
/// excerpt that happens to show an annotation cannot satisfy it. Returns the
/// exit status and the diagnostics as a person would read them.
fn assert_exact_diagnostics(
    relative: &str,
    profile: &str,
    root: &Path,
    expectations: &Expectations,
) -> (i32, String) {
    let checked = ember(&["check", "--json", relative, "--profile", profile], root);
    struct Produced {
        code: String,
        text: String,
        line: Option<usize>,
        rendered: String,
    }
    let mut produced = Vec::new();
    for line in checked.stdout.lines().chain(checked.stderr.lines()) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line.trim()) else { continue };
        let Some(code) = value["code"].as_str() else { continue };
        let mut text = value["message"].as_str().unwrap_or_default().to_string();
        let mut primary = None;
        for label in value["labels"].as_array().into_iter().flatten() {
            text.push('\n');
            text.push_str(label["message"].as_str().unwrap_or_default());
            if label["primary"].as_bool() == Some(true) && primary.is_none() {
                primary = label["span"]["line_start"].as_u64().map(|line| line as usize);
            }
        }
        for extra in ["notes", "helps"] {
            for item in value[extra].as_array().into_iter().flatten() {
                text.push('\n');
                text.push_str(item.as_str().unwrap_or_default());
            }
        }
        produced.push(Produced {
            code: code.to_string(),
            text,
            line: primary,
            rendered: value["rendered"].as_str().unwrap_or_default().to_string(),
        });
    }
    let expected = expectations
        .errors
        .iter()
        .zip(&expectations.error_lines)
        .chain(expectations.warnings.iter().zip(&expectations.warning_lines));
    let mut claimed = vec![false; produced.len()];
    let mut missing = Vec::new();
    for ((code, message), line) in expected {
        let found = (0..produced.len()).find(|&index| {
            let candidate = &produced[index];
            !claimed[index]
                && &candidate.code == code
                && candidate.text.contains(message.as_str())
                && line.is_none_or(|line| candidate.line == Some(line))
        });
        match found {
            Some(index) => claimed[index] = true,
            None => missing.push(match line {
                Some(line) => format!("{code}: {message:?} on line {line}"),
                None => format!("{code}: {message:?}"),
            }),
        }
    }
    let rendered: Vec<&str> = produced.iter().map(|p| p.rendered.as_str()).collect();
    let unexpected: Vec<&str> = produced
        .iter()
        .zip(&claimed)
        .filter(|(_, claimed)| !**claimed)
        .map(|(p, _)| p.rendered.as_str())
        .collect();
    assert!(
        missing.is_empty() && unexpected.is_empty(),
        "{relative} [{profile}]: [TST-1] the diagnostics differ from the annotations\nmissing: {missing:?}\nunexpected:\n{}\nall diagnostics:\n{}",
        unexpected.join("\n"),
        rendered.join("\n")
    );
    (checked.exit, rendered.join("\n"))
}

/// `ember` with `input` piped to its standard input (`#$ stdin:`).
/// `[TST-27]` — `ember` with the program's C compiled with warnings as errors, and `input` on its
/// standard input when given.
fn ember_gated(args: &[&str], root: &Path, input: Option<&[u8]>) -> Run {
    use std::io::Write;
    let (mut command, _output_directory) = test_command(args, root);
    command.env(ember_branding::c_warnings_as_errors_var(), "1");
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the ember binary runs");
    let stdin = child.stdin.take().expect("piped stdin");
    if let Some(input) = input {
        let mut stdin = stdin;
        stdin.write_all(input).expect("stdin is written");
    } else {
        drop(stdin);
    }
    let output = child.wait_with_output().expect("the ember binary finishes");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

fn ember_with_input(args: &[&str], root: &Path, input: &[u8]) -> Run {
    use std::io::Write;
    let (mut command, _output_directory) = test_command(args, root);
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the ember binary runs");
    child.stdin.take().expect("piped stdin").write_all(input).expect("stdin is written");
    let output = child.wait_with_output().expect("the ember binary finishes");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// `[CLI-20]`, `[VER-9]`, `[FMT-1]` — `ember fmt --migrate` rewrites source written for 0.9.8 into
/// 0.9.9 where the change is mechanical — `::` to `.`, integer `/` to `//`, `@view` and `with_views`
/// removal, `Box[dyn fn]` to an owned callable type, a pair loop over a `Map` to `m.items()` — and
/// lists what it changed, where the file has it; the result checks and is in the canonical layout.
#[test]
fn fmt_migrate_rewrites_the_mechanical_forms_of_0_9_8() {
    let root = workspace_root();
    let directory = temporary_directory("migrate");
    let old = directory.join(ember_branding::source_file("old"));
    std::fs::write(&old, "\
from std.borrow import with_views2

enum Color:
    Red
    Green

@view
struct Window:
    items: Span[int]

fn apply(f: Box[dyn fn(int) -> int], x: int) -> int:
    return f(x)

fn first_sum(a: Span[int], b: Span[int]) -> int:
    return with_views2(a, b, fn(x, y) => x[0] + y[0])

fn each(f: @latebound fn(Span[int]) -> int, s: Span[int]) -> int:
    return f(s)

fn main():
    _c = Color::Red
    x: int = 7
    y = x / 2
    y /= 2
    m: Map[str, int] = Map()
    m.insert(\"a\", 1)
    for k, v in m:
        println(k, v)
    xs: Array[int] = [1, 2]
    w = Window(xs[..])
    println(y, first_sum(xs[..], xs[..]), w.items.len())
").expect("the old source is writable");
    let old_path = old.to_string_lossy().to_string();
    let migrated = ember(&["fmt", "--migrate", &old_path], &root);
    assert_eq!(migrated.exit, 0, "{}", migrated.stderr);
    for wanted in [
        "fn apply(f: fn(int) -> int, x: int) -> int:",
        "return (fn(x: Span[i64], y: Span[i64]) => x[0] + y[0])(a, b)",
        "fn each(f: fn(Span[int]) -> int, s: Span[int]) -> int:",
        "_c = Color.Red",
        "y = x // 2",
        "y //= 2",
        "for (k, v) in m.items():",
    ] {
        assert!(migrated.stdout.contains(wanted), "the migration lacks {wanted:?}:\n{}", migrated.stdout);
    }
    for gone in ["::", "@view", "@latebound", "with_views", "Box[dyn", "std.borrow"] {
        assert!(!migrated.stdout.contains(gone), "the migration kept {gone:?}:\n{}", migrated.stdout);
    }
    // What changed, at the line and column of the file as written.
    for change in [
        ":1:1: `with_views` import removed",
        ":7:1: `@view` removed",
        ":11:21: `Box[dyn fn]` to an owned callable type",
        ":15:12: `with_views` call to a call of its callback",
        ":17:12: `@latebound` removed",
        ":21:15: `::` to `.`",
        ":23:9: integer `/` to `//`",
        ":24:5: integer `/` to `//`",
        ":27:9: `for k, v in m:` over a `Map` to `m.items()`",
    ] {
        assert!(migrated.stderr.contains(change), "the list lacks {change:?}:\n{}", migrated.stderr);
    }
    assert!(!migrated.stderr.contains("not rewritten"), "{}", migrated.stderr);
    // The result is a 0.9.9 program, already in the canonical layout.
    let new = directory.join(ember_branding::source_file("new"));
    std::fs::write(&new, &migrated.stdout).expect("the new source is writable");
    let new_path = new.to_string_lossy().to_string();
    let checked = ember(&["check", &new_path], &root);
    assert_eq!(checked.exit, 0, "the migrated program does not check:\n{}\n{}", checked.stderr, migrated.stdout);
    let formatted = ember(&["fmt", &new_path], &root);
    assert_eq!(formatted.stdout, migrated.stdout, "the migration is not in the canonical layout");
    // `--write` rewrites the file in place.
    let written = ember(&["fmt", "--migrate", "--write", &old_path], &root);
    assert_eq!(written.exit, 0, "{}", written.stderr);
    assert_eq!(std::fs::read_to_string(&old).expect("the file"), migrated.stdout);
}

/// `[CLI-20]` — what `ember fmt --migrate` cannot rewrite it lists, by where the file has it, and
/// the command fails; the mechanical part is still rewritten.
#[test]
fn fmt_migrate_lists_what_it_could_not_rewrite() {
    let root = workspace_root();
    let directory = temporary_directory("migrate");
    let old = directory.join(ember_branding::source_file("old"));
    std::fs::write(&old, "\
fn make() -> Box[dyn fn(int) -> int]:
    return Box(fn(x: int) => x + 1)

fn main():
    println(make()(1))
").expect("the old source is writable");
    let migrated = ember(&["fmt", "--migrate", &old.to_string_lossy()], &root);
    assert_ne!(migrated.exit, 0, "{}", migrated.stderr);
    assert!(migrated.stdout.contains("fn make() -> fn(int) -> int:"), "{}", migrated.stdout);
    assert!(migrated.stderr.contains(":1:22: `Box[dyn fn]` to an owned callable type"), "{}", migrated.stderr);
    let left: Vec<&str> = migrated.stderr.lines().filter(|line| line.starts_with("not rewritten: ")).collect();
    assert!(left.len() == 1 && left[0].contains(":2:"), "the boxed lambda is not listed:\n{}", migrated.stderr);
}

/// `[FMT-1]` — `ember fmt` writes at most 100 columns where brackets allow: a longer line breaks after
/// its last top-level opening bracket, one item per line with a trailing comma, again where a piece
/// is still too long, or else before each operator of its loosest kind, in parentheses put around a
/// statement's value; a long import list is parenthesised. The result formats to itself and parses
/// to the same tree. A `comptime:` block in a function keeps its body (it was flattened out of it),
/// a trailing comment stays on its line, a `#!` one at the end of the file too (on a line of its
/// own it was a misplaced directive), and a foreign import keeps its `with (…)` (it was dropped).
#[test]
fn fmt_keeps_lines_within_100_columns_and_changes_layout_only() {
    let root = workspace_root();
    let directory = temporary_directory("fmt");
    let source = directory.join(ember_branding::source_file("wide"));
    std::fs::write(&source, "\
from std.math import sqrt, floor, ceil, exp, log, sin, cos, tan, atan2, hypot, round, trunc, fma, pow
import c \"stdio.h\" with (overlay=\"stdio.overlay\")

fn f(a: int, b: int, c: int, d: int) -> int:
    return a + b + c + d

fn checked(n: int) -> int:
    return n    # the last line's comment

fn main():
    comptime:
        assert(checked(2) == 2)
    println(f(1111111111111, 2222222222222, 3333333333333, 4444444444444) + f(5555555555555, 6666666666666, 7777777777777, 8888888888888))
    xs: Array[int] = [1111111111111, 2222222222222, 3333333333333, 4444444444444, 5555555555555, 6666666666666, 77777777]
    total = xs[0] * 3 + xs[1] * 4 + xs[0] * xs[1] + xs[2] - xs[3] + xs[4] * xs[4] - xs[5] * xs[5] + xs[6] + 7
    if total > 1 and xs[0] > 2 and xs[1] + xs[2] > 3 and xs[3] * xs[4] > 4 and xs[5] - xs[6] < 5 and total > 6:
        println(total)
    println(xs.len())    #! a comment, not a directive
").expect("the source is writable");
    let path = source.to_string_lossy().to_string();
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    for line in once.stdout.lines() {
        assert!(line.chars().count() <= 100, "a line over 100 columns: {line:?}\n{}", once.stdout);
    }
    assert!(once.stdout.contains("    comptime:\n        assert(checked(2) == 2)\n"), "{}", once.stdout);
    assert!(once.stdout.contains("return n    # the last line's comment\n") || once.stdout.contains("return n # the last line's comment\n"), "{}", once.stdout);
    assert!(once.stdout.contains("println(xs.len()) #! a comment, not a directive"), "{}", once.stdout);
    assert!(once.stdout.contains("    xs: Array[int] = [\n        1111111111111,\n"), "{}", once.stdout);
    assert!(once.stdout.contains("    total = (\n        xs[0] * 3\n        + xs[1] * 4\n"), "{}", once.stdout);
    assert!(once.stdout.contains("    if (\n        total > 1\n        and xs[0] > 2\n"), "{}", once.stdout);
    assert!(once.stdout.contains("import c \"stdio.h\" with (overlay=\"stdio.overlay\")\n"), "{}", once.stdout);
    assert!(once.stdout.contains("from std.math import (\n    sqrt,\n    floor,\n"), "{}", once.stdout);
    let scratch = directory.join(ember_branding::source_file("once"));
    std::fs::write(&scratch, &once.stdout).expect("the scratch file is writable");
    let scratch_path = scratch.to_string_lossy().to_string();
    let twice = ember(&["fmt", &scratch_path], &root);
    assert_eq!(once.stdout, twice.stdout, "fmt(fmt(x)) differs from fmt(x)");
    let before = ember(&["build", &path, "--emit", "ast"], &root);
    let after = ember(&["build", &scratch_path, "--emit", "ast"], &root);
    assert_eq!(before.exit, 0, "{}", before.stderr);
    assert_eq!(before.stdout, after.stdout, "parse(fmt(x)) differs from parse(x)");
}

/// `[FMT-1]` — every declaration is laid out, none copied through as written: a class (its base,
/// interfaces and members), an `unsafe extern` block (its attributes, `safe fn`s and comments), an
/// `extern class` and an item-level `comptime:` block are re-indented to four spaces. The result
/// formats to itself and parses to the same tree.
#[test]
fn fmt_lays_out_classes_extern_blocks_and_comptime_items() {
    let root = workspace_root();
    let directory = temporary_directory("fmt-items");
    let source = directory.join(ember_branding::source_file("items"));
    std::fs::write(&source, "\
interface Named:
  fn name(self) -> int

open class Base:
  id: int = 0

  fn get(self) -> int:
        return self.id

class Leaf(Base) implements Named:
  extra: int = 1
  fn name(self) -> int:
        return self.extra

unsafe extern \"C\":
  # the C library's absolute value
  @ffi(effects=[], deterministic)
  safe fn abs(x: i32) -> i32
  fn rand() -> i32

extern class cpp.RageV.Layer:
  pass

comptime:
      assert(1 + 1 == 2)

fn main():
    leaf = Leaf()
    println(leaf.name(), abs(-3))
").expect("the source is writable");
    let path = source.to_string_lossy().to_string();
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    let out = &once.stdout;
    assert!(out.contains("open class Base:\n    id: int = 0\n\n    fn get(self) -> int:\n        return self.id\n"), "{out}");
    assert!(out.contains("class Leaf(Base) implements Named:\n    extra: int = 1\n\n    fn name(self) -> int:\n        return self.extra\n"), "{out}");
    assert!(out.contains("unsafe extern \"C\":\n    # the C library's absolute value\n    @ffi(effects=[], deterministic)\n    safe fn abs(x: i32) -> i32\n    fn rand() -> i32\n"), "{out}");
    assert!(out.contains("extern class cpp.RageV.Layer:\n    pass\n"), "{out}");
    assert!(out.contains("comptime:\n    assert(1 + 1 == 2)\n"), "{out}");
    let scratch = directory.join(ember_branding::source_file("once"));
    std::fs::write(&scratch, &once.stdout).expect("the scratch file is writable");
    let scratch_path = scratch.to_string_lossy().to_string();
    let twice = ember(&["fmt", &scratch_path], &root);
    assert_eq!(once.stdout, twice.stdout, "fmt(fmt(x)) differs from fmt(x)");
    let before = ember(&["build", &path, "--emit", "ast"], &root);
    let after = ember(&["build", &scratch_path, "--emit", "ast"], &root);
    assert_eq!(before.stdout, after.stdout, "parse(fmt(x)) differs from parse(x)");
}

/// `[MONO-2]` — `ember build --report=instantiations` prints, per generic, how many instances it
/// produced (a function's and a type's alike), each in the order made, and the time checking them
/// took. `[MONO-3]` — `[build] max_instantiations = N` is a per-generic ceiling: past it, `W2220`
/// names the generic and its newest instances. The standard library's generics are not the
/// package's to budget (ODR-135), and a ceiling that is not a positive whole number is `E9034`.
#[test]
fn instantiations_are_reported_and_a_ceiling_warns() {
    let root = workspace_root();
    let directory = temporary_directory("instantiations");
    std::fs::create_dir_all(directory.join("src")).expect("the package is writable");
    std::fs::write(directory.join(MANIFEST), "[package]\nname = \"mono\"\n\n[build]\nmax_instantiations = 2\n")
        .expect("the manifest is writable");
    std::fs::write(
        directory.join("src").join(ember_branding::source_file("main")),
        "fn pick[T](a: T, b: T) -> T:\n    return b\n\n\
         struct Wrap[T]:\n    value: T\n\n\
         fn main():\n    println(pick(1, 2), pick(1.5, 2.5), pick(\"a\", \"b\"))\n\
         \x20   w = Wrap[int](value = 3)\n    v = Wrap[bool](value = true)\n    println(w.value, v.value)\n",
    )
    .expect("the source is writable");
    let package = directory.to_string_lossy().into_owned();
    let out = directory.join("out").to_string_lossy().into_owned();
    let built = ember(&["build", "--report=instantiations", &package, "--out-dir", &out], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    assert!(built.stdout.contains("  pick: 3 instance(s), "), "{}", built.stdout);
    assert!(built.stdout.contains("    pick[i64]\n    pick[f64]\n    pick[str]\n"), "{}", built.stdout);
    assert!(built.stdout.contains("  Wrap: 2 instance(s), ") && built.stdout.contains("    Wrap[i64]\n    Wrap[bool]\n"), "{}", built.stdout);
    assert!(built.stderr.contains("warning[W2220]: `pick` has 3 instances, more than `max_instantiations = 2`"), "{}", built.stderr);
    assert!(built.stderr.contains("its newest: pick[str], pick[f64], pick[i64]"), "{}", built.stderr);
    assert_eq!(built.stderr.matches("warning[W2220]").count(), 1, "only the program's own generics: {}", built.stderr);
    std::fs::write(directory.join(MANIFEST), "[package]\nname = \"mono\"\n\n[build]\nmax_instantiations = 0\n")
        .expect("the manifest is writable");
    let refused = ember(&["check", &package], &root);
    assert_ne!(refused.exit, 0);
    assert!(refused.stderr.contains("error[E9034]") && refused.stderr.contains("is not a positive whole number"), "{}", refused.stderr);
}

/// `[TCB-4]` — `ember tcb` lists the trusted base by category: the compiler with its version and
/// the defects its ledger lists as open, the runtime, the standard library, the C++ bridge, external
/// libraries, the driver and the hardware, then each `unsafe` block and function with its note.
/// `[CLI-11]` — `ember audit` prints a one-page summary first: the `unsafe` code, how many of the
/// program's functions carry each effect, and the foreign boundary.
#[test]
fn tcb_lists_each_category_and_audit_summarises() {
    let root = workspace_root();
    let dir = temporary_directory("audit");
    let source = dir.join(format!("audit.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "unsafe extern \"C\":\n    @ffi(effects=[], deterministic)\n    safe fn abs(x: i32) -> i32\n    fn rand() -> i32\n\n\
         fn grow(mut xs: Array[int]):\n    xs.push(1)\n\n\
         fn noisy() -> i32:\n    unsafe:  # SAFETY(ffi): rand has no precondition\n        return rand()\n\n\
         fn main():\n    xs: Array[int] = []\n    grow(xs)\n    println(abs(-2), noisy())\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let tcb = ember(&["tcb", &source_arg], &root);
    assert_eq!(tcb.exit, 0, "{}", tcb.stderr);
    for category in ["Compiler: ember ", "Runtime: ", "Standard library: ", "C++ bridge: ", "External libraries: ", "Driver: ", "Hardware: "] {
        assert!(tcb.stdout.contains(category), "[TCB-4] no `{category}` line:\n{}", tcb.stdout);
    }
    assert!(tcb.stdout.contains("known defects: D-"), "[TCB-4] the ledger's open defects:\n{}", tcb.stdout);
    assert!(tcb.stdout.contains("block in `noisy`  SAFETY(ffi): rand has no precondition"), "{}", tcb.stdout);
    let audit = ember(&["audit", &source_arg], &root);
    assert_eq!(audit.exit, 0, "{}", audit.stderr);
    let out = &audit.stdout;
    assert!(out.starts_with("Audit: "), "{out}");
    assert!(out.contains("  unsafe: 1 block(s) and 0 unsafe fn(s), 1 with a safety note"), "{out}");
    assert!(out.contains("  effects (functions carrying each): Alloc "), "{out}");
    assert!(out.contains("Io "), "{out}");
    assert!(out.contains("  foreign boundary: 2 declaration(s), 1 safe to call"), "{out}");
    assert!(out.contains("Trusted base: ") && out.contains("Compiler: ember "), "the trusted base follows:\n{out}");
    let json = ember(&["audit", "--json", &source_arg], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("audit --json is JSON");
    assert!(value["effects"]["Alloc"].as_u64().is_some_and(|count| count >= 2), "{}", json.stdout);
    assert!(value["categories"]["compiler"].as_str().is_some_and(|text| text.contains("known defects")), "{}", json.stdout);
}

/// `[COST-4]` — `ember inspect --cost` prints every row of the cost table that applies, each with
/// its class (`[COST-2]`: conditionally elided or guaranteed present here) and its sites, and for a
/// conditionally elided cost whether each site's was removed and by which proof. `[PHIL-15]` — the
/// costs the language inserts unwritten are each reported by site: bounds and overflow checks, a
/// count operation, a dynamic exclusivity check, an indirect call, an allocation. `[STD-8a]` — which
/// `contains` an `x in c` selects: a scan of an `Array`, linear; a `Set`'s, constant.
#[test]
fn inspect_cost_reports_each_row_by_site() {
    let root = workspace_root();
    let dir = temporary_directory("inspect-cost");
    let source = dir.join(format!("costs.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "from std.collections import Set\n\n\
         interface Shape:\n    fn area(self) -> int\n\n\
         struct Square:\n    side: int\n\n\
         extend Square implements Shape:\n    fn area(self) -> int:\n        return self.side * self.side\n\n\
         class Counter:\n    n: int = 0\n\n    fn bump(mut self):\n        self.n += 1\n\n\
         fn bump_all(counters: Array[Counter]):\n    for c in counters:\n        keep = c\n        keep.bump()\n\n\
         fn measure(shape: ref dyn Shape) -> int:\n    return shape.area()\n\n\
         fn get(xs: Array[int], i: int) -> int:\n    return xs[i]\n\n\
         fn main():\n    xs = [1, 2, 3]\n    s = Set[int]()\n    s.add(2)\n    text = f\"value {xs[0]}\"\n\
         \x20   square = Square(side = 2)\n    counters = [Counter(), Counter()]\n    bump_all(counters)\n\
         \x20   println(get(xs, 1), 2 in xs, 2 in s, \"lu\" in text, measure(ref square), counters[0].n)\n",
    )
    .expect("write the program");
    let out_dir = dir.join("out");
    let build = ember(&["build", &source.to_string_lossy(), "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(build.exit, 0, "build failed:\n{}", build.stderr);
    let table = out_dir.join("debug").join("inspect").join("costs.safety.json").to_string_lossy().into_owned();
    let report = ember(&["inspect", &table, "--cost"], &root);
    assert_eq!(report.exit, 0, "{}", report.stderr);
    let out = &report.stdout;
    for row in [
        "  Bounds check on `a[i]` [conditionally elided]",
        "  Overflow check on integer arithmetic [conditionally elided]",
        "  Dynamic exclusivity check [conditionally elided]",
        "  Allocation [guaranteed present]",
        "  Interface call through `dyn` [guaranteed present]",
        "  Retain / release, surviving [guaranteed present]",
        "  `contains` [guaranteed present]",
    ] {
        assert!(out.contains(row), "no row `{row}`:\n{out}");
    }
    assert!(out.contains("  main  present: index test"), "{out}");
    assert!(out.contains("removed: range facts (range_facts)"), "{out}");
    assert!(out.contains("  bump_all  present: dynamic exclusivity") || out.contains("removed: static exclusivity"), "{out}");
    assert!(out.contains("an f-string"), "{out}");
    assert!(out.contains("  measure  one indirect call"), "{out}");
    assert!(out.contains("  bump_all  a retain in a loop"), "{out}");
    assert!(out.contains("a scan of the `Array[i64]`, element by element: linear in its length"), "{out}");
    assert!(out.contains(".contains`: constant on average: one hash and its probe"), "{out}");
    assert!(out.contains("`str.contains`: a search of the text: linear in the text's length"), "{out}");
    assert!(!out.contains(&format!("collections.{SOURCE_EXT}")), "the standard library's own sites are its own:\n{out}");
    let measure = ember(&["inspect", &table, "--cost", "--function", "measure"], &root);
    assert!(measure.stdout.contains("Interface call through `dyn`") && !measure.stdout.contains("Allocation"), "{}", measure.stdout);
    let json = ember(&["inspect", &table, "--cost", "--json"], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("--cost --json is JSON");
    let rows = value["costs"].as_array().expect("the rows");
    assert!(rows.iter().any(|row| row["row"] == "Allocation" && row["class"] == "guaranteed present"), "{}", json.stdout);
}

/// `[CLI-10]` — `ember build --report=engine` is a report, not a profile: the program's C is the
/// same with it as without, and it summarises the program's own allocations, the count operations
/// surviving in loops, the runtime checks kept with their reasons (and how many a proof removed),
/// the C compiler's vectorisation remarks, each foreign function's wrapper, the inlining decisions,
/// and the classes that could be structs (advisory).
#[test]
fn the_engine_report_summarises_the_build_and_changes_nothing() {
    let root = workspace_root();
    let directory = temporary_directory("engine-report");
    let file = directory.join(ember_branding::source_file("engine"));
    std::fs::write(&file, "class Counter:\n    count: int\n\n    fn init(mut self):\n        self.count = 0\n\n    fn bump(mut self):\n        self.count += 1\n\nunsafe extern \"C\":\n    @ffi(param(xs, borrowed, count(n)), link_name=\"sum_c\")\n    safe fn sum(xs: Span[i32], n: usize) -> i32\n\npub extern \"C\" fn sum_c(xs: *i32, n: usize) -> i32:\n    total: i32 = 0\n    i: usize = 0\n    while i < n:\n        unsafe:  # SAFETY(ffi): C's contract: `xs` holds `n` elements\n            total += read(xs, i)\n        i += 1\n    return total\n\nfn pick(xs: Array[int], i: int) -> int:\n    return xs[i]\n\n@noinline\nfn twice(x: int) -> int:\n    return x * 2\n\nfn main():\n    xs: Array[int] = [1, 2, 3]\n    ys: Array[i32] = [4, 5, 6]\n    c = Counter()\n    for _x in xs:\n        c.bump()\n    println(pick(xs, 1), twice(c.count), sum(ys.as_span()))\n").expect("the program is writable");
    let file = file.to_string_lossy().into_owned();
    let out = directory.join("out").to_string_lossy().into_owned();
    let built = ember(&["build", &file, "--report=engine", "--out-dir", &out], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    let report = &built.stdout;
    for line in [
        "engine report (a report: nothing in it changes the program)",
        "allocations (3):",
        "in main: a new class instance",
        "count operations surviving in loops (0):",
        "in main: Bounds (index test): not_proven_by_analysis",
        "in twice: Overflow (overflow test): not_proven_by_analysis",
        "loops (the C compiler's vectorisation remarks,",
        "sum (`sum_c`): one call, plus a view as its pointer; a length checked to fit",
        "pick inlined into main: called from that one place",
        "twice: `@noinline`: never inlined",
        "Counter (",
        "final, with no base and no virtual method, and no handle of it is ever copied",
    ] {
        assert!(report.contains(line), "the engine report lacks `{line}`:\n{report}");
    }
    assert!(!report.contains("std."), "the report names the standard library's code:\n{report}");
    let c = |report: bool| {
        let mut args = vec!["build", file.as_str(), "--emit", "c", "--out-dir", out.as_str()];
        if report {
            args.push("--report=engine");
        }
        let emitted = ember(&args, &root);
        assert_eq!(emitted.exit, 0, "{}", emitted.stderr);
        std::fs::read_to_string(directory.join("out").join("debug").join("c").join("engine.c")).expect("the C was written")
    };
    assert_eq!(c(false), c(true), "`--report=engine` changed the program's C");
}

/// `[DIA-8]` — `ember explain --borrow <file>:<line>` prints, for each loan live at that line, where
/// it was created, the last line its region covers and the later use that keeps it alive, from the
/// borrow checker's own loans (`[BCK-1]`): a shared borrow kept by a later call, a mutable one kept
/// by a later argument, a loan created on the line asked about, and a line with none.
#[test]
fn explain_borrow_names_each_live_loan_and_the_use_that_keeps_it() {
    let root = workspace_root();
    let directory = temporary_directory("explain-borrow");
    let shared = directory.join(ember_branding::source_file("shared"));
    std::fs::write(&shared, "fn total(xs: Span[int]) -> int:\n    sum = 0\n    for x in xs:\n        sum += x\n    return sum\n\nfn main():\n    v: Array[int] = [1, 2, 3]\n    s = v.as_span()\n    n = v.len()\n    w = total(s)\n    println(w, n)\n")
        .expect("the program is writable");
    let exclusive = directory.join(ember_branding::source_file("exclusive"));
    std::fs::write(&exclusive, "fn bump(mut xs: Array[int]):\n    xs.push(4)\n\nfn main():\n    v: Array[int] = [1, 2, 3]\n    r = ref mut v\n    r.push(9)\n    bump(r)\n    println(v.len())\n")
        .expect("the program is writable");
    let explain = |file: &Path, line: u32| {
        let run = ember(&["explain", "--borrow", &format!("{}:{line}", file.display())], &root);
        assert_eq!(run.exit, 0, "{}", run.stderr);
        run.stdout
    };
    let kept = "borrow of v created at 9:9, live through 11, because s is used at 11:15\n";
    assert_eq!(explain(&shared, 10), kept);
    assert_eq!(explain(&shared, 9), kept, "a loan created on the line is live there");
    assert_eq!(explain(&shared, 12), "no borrow is live at line 12\n");
    let mutable = explain(&exclusive, 6);
    assert_eq!(mutable, "mutable borrow of v created at 6:9, live through 8, because r is used at 8:5\n");
    assert!(explain(&exclusive, 8).starts_with("mutable borrow of v created at 6:9, live through 8, because r is used at 8:5\n"));
}

/// `[DOC-1]` — error pages ship with their errors: `ember explain <code>` prints the page from the
/// compiler itself, wherever it runs, not from a repository it may not be in.
#[test]
fn explain_prints_the_page_built_into_the_compiler() {
    let root = workspace_root();
    let elsewhere = temporary_directory("explain-elsewhere");
    let shown = ember(&["explain", "E9034"], &elsewhere);
    assert_eq!(shown.exit, 0, "{}", shown.stderr);
    let page = std::fs::read_to_string(root.join("docs").join("errors").join("E9034.md")).expect("the page");
    assert!(shown.stdout.contains(page.lines().next().expect("a title")), "{}", shown.stdout);
    assert!(shown.stdout.contains("## The fix"), "{}", shown.stdout);
    let json = ember(&["explain", "E9034", "--json"], &elsewhere);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("explain --json is JSON");
    assert!(value["page"].as_str().is_some_and(|text| text.contains("## Why")), "{}", json.stdout);
}

/// `[CLI-15]` — `ember --help` lists every command and flag of XVII.1: each `ember <command>` and each
/// `--flag` (and `-D`) of the specification's listing, read from the development target itself, is in it.
#[test]
fn help_lists_every_command_and_flag_of_the_specification() {
    let root = workspace_root();
    let target = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the target");
    let path = target.split("\"path\": \"").nth(1).and_then(|rest| rest.split('"').next()).expect("its path");
    let spec = std::fs::read_to_string(root.join(path)).expect("the specification");
    let section = spec.split("## XVII.1 The `ember` command").nth(1).expect("XVII.1");
    let listing = section.split("```text").nth(1).and_then(|rest| rest.split("```").next()).expect("its listing");
    let help = ember(&["--help"], &root);
    let shown = format!("{}{}", help.stdout, help.stderr);
    let mut wanted = Vec::new();
    for line in listing.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        if let Some(at) = words.iter().position(|word| *word == "ember") {
            if let Some(command) = words.get(at + 1).filter(|word| word.chars().next().is_some_and(char::is_alphabetic)) {
                wanted.push(format!("ember {command}"));
            }
        }
        for word in &words {
            for piece in word.split(['|', '[', ']']) {
                if piece.starts_with("--") || piece == "-D" {
                    let flag: String = piece.chars().take_while(|c| c.is_alphanumeric() || *c == '-').collect();
                    if flag.len() > 2 {
                        wanted.push(flag);
                    }
                }
            }
        }
    }
    assert!(wanted.len() > 40, "the listing was not read: {wanted:?}");
    let missing: Vec<&String> = wanted.iter().filter(|word| !shown.contains(word.as_str())).collect();
    assert!(missing.is_empty(), "`ember --help` omits {missing:?}:\n{shown}");
}

fn ember(args: &[&str], root: &Path) -> Run {
    let (mut command, _output_directory) = test_command(args, root);
    let output = command
        .output()
        .expect("the ember binary runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

fn ember_with_env(args: &[&str], root: &Path, key: &str, value: &str) -> Run {
    let (mut command, _output_directory) = test_command(args, root);
    let output = command
        .env(key, value)
        .output()
        .expect("the ember binary runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// Commands whose artifacts are not inspected afterwards get a private output
/// directory. Explicit --out-dir callers keep their own directory guard alive
/// until they have finished inspecting the generated C, headers and binaries.
fn test_command(args: &[&str], root: &Path) -> (Command, Option<TemporaryDirectory>) {
    let mut command = Command::new(EMBER);
    command.args(args).current_dir(root);
    let directory = if matches!(args.first().copied(), Some("build" | "run" | "check" | "test"))
        && !args.contains(&"--out-dir")
    {
        let directory = temporary_directory("cli");
        command.arg("--out-dir").arg(&directory);
        Some(directory)
    } else {
        None
    };
    (command, directory)
}

fn compile_cpp_object(
    toolchain: &Toolchain,
    source: &Path,
    object: &Path,
    include_dirs: &[PathBuf],
) -> Result<(), String> {
    let mut command = match toolchain {
        Toolchain::Msvc { cl, env } => {
            let mut command = Command::new(cl);
            command.envs(env);
            command.args(["/nologo", "/TP", "/std:c++17", "/c"]);
            for include_dir in include_dirs {
                command.arg(format!("/I{}", include_dir.display()));
            }
            command.arg(source).arg(format!("/Fo{}", object.display()));
            command
        }
        Toolchain::Clang(path) => {
            let mut command = Command::new(path);
            command.args(["-x", "c++", "-std=c++17", "-Wall", "-Wextra"]);
            for include_dir in include_dirs {
                command.arg("-I").arg(include_dir);
            }
            command.args(["-c"]).arg(source).arg("-o").arg(object);
            command
        }
        Toolchain::Gcc(path) => {
            let mut command = Command::new(path);
            command.args(["-x", "c++", "-std=c++17", "-Wall", "-Wextra"]);
            for include_dir in include_dirs {
                command.arg("-I").arg(include_dir);
            }
            command.args(["-c"]).arg(source).arg("-o").arg(object);
            command
        }
    };
    let output = command.output().map_err(|error| format!("could not run C++ compiler: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "C++ header translation unit failed:\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

#[test]
fn temporary_test_outputs_are_removed_when_the_case_finishes() {
    let path = {
        let directory = temporary_directory("cleanup-normal");
        let nested = directory.join("release/bin");
        std::fs::create_dir_all(&nested).expect("test output directory is writable");
        std::fs::write(nested.join("program"), b"compiled test output").expect("test output is writable");
        directory.to_path_buf()
    };
    let removed = !path.exists();
    if !removed {
        std::fs::remove_dir_all(&path).expect("remove the regression test's own failed output");
    }
    assert!(removed, "a completed test must not retain its compiled programs");
}

#[test]
fn temporary_test_outputs_are_removed_when_the_case_panics() {
    let directory = temporary_directory("cleanup-panic");
    let path = directory.to_path_buf();
    std::fs::write(directory.join("program"), b"compiled test output").expect("test output is writable");
    let outcome = std::panic::catch_unwind(move || {
        let _directory = directory;
        panic!("simulated failing test");
    });
    assert!(outcome.is_err());
    let removed = !path.exists();
    if !removed {
        std::fs::remove_dir_all(&path).expect("remove the regression test's own failed output");
    }
    assert!(removed, "a failing test must not retain its compiled programs");
}

/// `[BLD-13]`, `[DET-7]` — the same program built in two folders, each from its own, is the same
/// bytes in every profile and holds no absolute path of either folder or of the repository (the
/// standard library and the runtime); `ember build --build-id` prints the same hash of the inputs
/// for both, and another once the source changes. The global program cache is off, so each build
/// is compiled and linked.
#[test]
fn a_build_is_reproducible_and_named_by_its_inputs() {
    let directory = temporary_directory("reproducible");
    let source = "fn pick(xs: Array[int], i: int) -> int:\n    return xs[i]\n\nfn main():\n    xs: Array[int] = [1, 2, 3]\n    println(pick(xs, 1), f\"{xs}\")\n";
    let folders = [directory.join("alpha"), directory.join("beta").join("nested")];
    let input = ember_branding::source_file("main");
    for folder in &folders {
        std::fs::create_dir_all(folder).expect("a build folder is writable");
        std::fs::write(folder.join(&input), source).expect("the program is writable");
    }
    // `compiler/ember_driver`'s grandparent, as the build spells it (not `canonicalize`'s `\\?\` form).
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("the repository").to_path_buf();
    let holds = |bytes: &[u8], text: &str| bytes.windows(text.len()).any(|window| window == text.as_bytes());
    let no_cache = ember_branding::no_compile_cache_var();
    let mut first_id = String::new();
    for profile in [Profile::Debug, Profile::Development, Profile::Release] {
        let mut ids = Vec::new();
        let mut programs = Vec::new();
        for folder in &folders {
            let built = ember_with_env(
                &["build", &input, "--profile", profile.name(), "--out-dir", "out", "--build-id"],
                folder, &no_cache, "1",
            );
            assert_eq!(built.exit, 0, "{}: {}", profile.name(), built.stderr);
            let id = built.stdout.lines().next().unwrap_or_default().to_string();
            assert!(id.len() == 64 && id.bytes().all(|byte| byte.is_ascii_hexdigit()), "{}: build id `{id}`", profile.name());
            ids.push(id);
            let program = folder.join("out").join(profile.name()).join("bin")
                .join(format!("main{}", std::env::consts::EXE_SUFFIX));
            let bytes = std::fs::read(&program).expect("the program was built");
            for place in [folder.as_path(), repository.as_path()] {
                let shown = place.display().to_string();
                for spelling in [shown.clone(), shown.replace('\\', "/")] {
                    assert!(!holds(&bytes, &spelling), "{}: the program holds the absolute path `{spelling}`", profile.name());
                }
            }
            programs.push(bytes);
        }
        assert_eq!(ids[0], ids[1], "{}: the same inputs give another build id", profile.name());
        assert!(programs[0] == programs[1], "{}: the same inputs built in two folders differ", profile.name());
        if profile == Profile::Debug {
            first_id = ids[0].clone();
        }
    }
    std::fs::write(folders[0].join(&input), source.replace("[1, 2, 3]", "[1, 2, 4]")).expect("the program is writable");
    let changed = ember_with_env(&["build", &input, "--out-dir", "out", "--build-id"], &folders[0], &no_cache, "1");
    assert_eq!(changed.exit, 0, "{}", changed.stderr);
    assert_ne!(changed.stdout.lines().next().unwrap_or_default(), first_id, "a changed source keeps its build id");
}

/// A managed executable may discard unreachable implementation code, but a
/// defined C entry point remains callable even without an Ember caller. Its
/// selected symbol, including imported and relaxed-FP definitions, is a root;
/// an unused foreign declaration is not a definition the linker must find.
#[cfg(target_os = "linux")]
#[test]
fn gcc_executables_keep_c_exports_and_discard_unreachable_native_bodies() {
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    let Toolchain::Gcc(compiler) = &toolchain else { return; };
    let root = workspace_root();
    let directory = temporary_directory("executable-roots");
    // Check actual linker acceptance independently of the production detector.
    // An always-false detector must not silently skip all removal assertions.
    let probe = directory.join("probe.c");
    std::fs::write(&probe,
        "int probe_c_root(void) { return 42; }\nint main(void) { return 0; }\n").unwrap();
    let probe_output = Command::new(compiler)
        .args(["-ffunction-sections", "-fdata-sections", "-Xlinker", "--gc-sections",
            "-Xlinker", "--require-defined=probe_c_root"])
        .arg(&probe).arg("-o").arg(directory.join("probe"))
        .output().expect("linker capability probe runs");
    let gc_available = probe_output.status.success();
    assert_eq!(ember_build::executable_section_gc_supported(&toolchain), gc_available,
        "linker capability detection disagrees with an actual link");
    let imported = directory.join(ember_branding::source_file("exports"));
    std::fs::write(&imported,
        "@export(\"imported_root\", threads=any)\npub fn imported_value() -> i32:\n    return 42\n")
        .expect("imported exports are writable");
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&entry,
        "import exports\n@export(\"selected_root\", threads=any)\npub fn local_value() -> i32:\n    return 43\npub extern \"C\" fn plain_root() -> i32:\n    return 44\n@export(\"relaxed_root\", threads=any)\n@fp(contract)\npub fn relaxed_value(x: f64) -> f64:\n    return x * x + 1.0\nunsafe extern \"C\":\n    fn absent_foreign() -> i32\n@noinline\npub fn unused_native() -> i32:\n    return 99\nfn main():\n    println(7)\n")
        .expect("executable fixture is writable");
    let entry_arg = entry.to_string_lossy().into_owned();
    // A file cannot contain cache/runtime: this exercises the uncached runtime
    // source fallback without depending on filesystem permissions or root uid.
    let unavailable_cache = directory.join("unavailable-cache");
    std::fs::write(&unavailable_cache, "not a directory").unwrap();
    let cache_var = ember_branding::cache_dir_var();
    for profile in [Profile::Debug, Profile::Development, Profile::Release] {
        for cached in [true, false] {
            let output_dir = directory.join(format!("{}-{cached}", profile.name()));
            let output_arg = output_dir.to_string_lossy().into_owned();
            let cache = if cached { directory.join("cache") } else { unavailable_cache.clone() };
            let built = ember_with_env(
                &["build", &entry_arg, "--profile", profile.name(), "--out-dir", &output_arg],
                &root, &cache_var, &cache.to_string_lossy(),
            );
            assert_eq!(built.exit, 0, "{} cached={cached}: {}", profile.name(), built.stderr);
            let executable = output_dir.join(profile.name()).join("bin/entry");
            let run = ember_build::output_started(&mut Command::new(&executable)).expect("managed executable runs");
            assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
            assert_eq!(run.stdout, b"7\n");
            assert!(run.stderr.is_empty());
            let listed = Command::new("nm").arg("--defined-only").arg(&executable)
                .output().expect("ELF symbol reader runs");
            assert!(listed.status.success());
            let listing = String::from_utf8_lossy(&listed.stdout);
            let symbols: Vec<_> = listing.lines().filter_map(|line| line.split_whitespace().last()).collect();
            for symbol in ["imported_root", "selected_root", "plain_root", "relaxed_root"] {
                assert!(symbols.contains(&symbol), "{} cached={cached}: missing C definition {symbol}", profile.name());
            }
            assert_eq!(symbols.contains(&ember_branding::mangled("unused_native").as_str()), !gc_available,
                "{} cached={cached}: unreachable native body survived or fallback lost it", profile.name());
            assert!(!symbols.contains(&"absent_foreign"));
        }
    }
}

/// `[FFI-10]` — a hand-declared static names linker storage rather than an
/// inlined Ember constant. A separate C translation unit proves the actual
/// symbol binding, including `link_name` and `@ffi(immutable)`.
#[test]
fn foreign_scalar_statics_read_the_linkers_storage() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-10/accept_foreign_scalar_statics.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "foreign static C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains("extern int32_t raw_counter;"));
    assert!(emitted.stdout.contains("extern int32_t changing_counter;"));
    assert!(emitted.stdout.contains("extern const int32_t frozen_count;"));
    assert!(emitted.stdout.contains("extern const int32_t aliased_count;"));

    let directory = temporary_directory("foreign-static");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_statics.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let output = directory.join(if cfg!(windows) { "foreign_static.exe" } else { "foreign_static" });
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture.clone(), runtime_source.clone()],
        include_dirs: &includes,
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("foreign static and C fixture link together");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "foreign static program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "41\n7\n9\n10\n22\n484\n1\n2\n");

    // Module imports must retain the foreign binding instead of folding it
    // like an ordinary `const` or losing its C symbol at the boundary.
    let imported = directory.join(ember_branding::source_file("imported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "unsafe extern \"C\":\n    @ffi(immutable, link_name=\"aliased_count\")\n    pub static renamed_count: i32\n    pub static mut changing_counter: i32\n    pub safe fn read_changing_counter() -> i32\n")
        .expect("foreign static module is writable");
    for (label, source) in [
        ("from_import", "from imported import renamed_count, changing_counter, read_changing_counter\nfn main():\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        changing_counter = 12\n    println(read_changing_counter())\n    println(renamed_count)\n"),
        ("qualified", "import imported\nfn main():\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        imported.changing_counter = 12\n    println(imported.read_changing_counter())\n    println(imported.renamed_count)\n"),
    ] {
        std::fs::write(&entry, source).expect("importing module is writable");
        let entry_arg = entry.to_string_lossy().into_owned();
        let across_modules = ember(&["build", &entry_arg, "--emit", "c"], &root);
        assert_eq!(across_modules.exit, 0, "{label} foreign static failed: {}", across_modules.stderr);
        assert!(across_modules.stdout.contains("extern const int32_t aliased_count;"));
        assert!(across_modules.stdout.contains("extern int32_t changing_counter;"));
        let imported_c = directory.join(format!("{label}.c"));
        std::fs::write(&imported_c, across_modules.stdout).expect("imported C is writable");
        let imported_exe = directory.join(if cfg!(windows) { format!("{label}.exe") } else { label.to_string() });
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[imported_c, fixture.clone(), runtime_source.clone()],
            include_dirs: &includes,
            output: imported_exe.clone(),
            profile: Profile::Debug,
            obj_dir: directory.join("obj"),
        }).expect("imported static links to the C fixture");
        let imported_run = ember_build::output_started(&mut Command::new(imported_exe)).expect("imported static program runs");
        assert!(imported_run.status.success());
        assert_eq!(String::from_utf8_lossy(&imported_run.stdout).replace("\r\n", "\n"), "12\n2\n");
    }
}

/// `[FFI-8]` — a plain C record global is copied by value, and a mutable
/// declaration writes back to the same linker object that C reads.
#[test]
fn foreign_record_statics_exchange_values_with_c() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-10/accept_foreign_record_statics.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "foreign record C emission failed: {}", emitted.stderr);

    let directory = temporary_directory("foreign-record-static");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_statics.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "foreign_record.exe" } else { "foreign_record" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture.clone(), runtime_source.clone()],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("foreign record and C fixture link together");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "foreign record program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "11\n2\n7\n17\n21\n35\n16\n24\n37\n");

    let imported = directory.join(ember_branding::source_file("imported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignPair:\n    pub left: i32\n    pub right: i32\n@derive(Copy)\npub struct ForeignInner:\n    pub value: i32\n@derive(Copy)\npub struct ForeignOuter:\n    pub inner: ForeignInner\n    pub other: i32\nunsafe extern \"C\":\n    pub static mut shared_pair: ForeignPair\n    @ffi(immutable)\n    pub static frozen_pair: ForeignPair\n    pub safe fn read_pair_sum() -> i32\n    pub static mut nested_pair: ForeignOuter\n    pub safe fn read_nested_total() -> i32\n")
        .expect("foreign record module is writable");
    for (label, source) in [
        ("from_import", "from imported import ForeignPair, shared_pair, frozen_pair, read_pair_sum, nested_pair, read_nested_total\nfn main():\n    println(frozen_pair.left)\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        shared_pair = ForeignPair(left = 11, right = 12)\n        println(read_pair_sum())\n        shared_pair.left = 20\n        println(read_pair_sum())\n        shared_pair.right += 2\n        println(read_pair_sum())\n        nested_pair.inner.value = 15\n        println(read_nested_total())\n        nested_pair.inner.value += 2\n        println(read_nested_total())\n"),
        ("qualified", "import imported\nfn main():\n    println(imported.frozen_pair.left)\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        imported.shared_pair = imported.ForeignPair(left = 11, right = 12)\n        println(imported.read_pair_sum())\n        imported.shared_pair.left = 20\n        println(imported.read_pair_sum())\n        imported.shared_pair.right += 2\n        println(imported.read_pair_sum())\n        imported.nested_pair.inner.value = 15\n        println(imported.read_nested_total())\n        imported.nested_pair.inner.value += 2\n        println(imported.read_nested_total())\n"),
    ] {
        std::fs::write(&entry, source).expect("foreign record entry is writable");
        let entry_arg = entry.to_string_lossy().into_owned();
        let across_modules = ember(&["build", &entry_arg, "--emit", "c"], &root);
        assert_eq!(across_modules.exit, 0, "{label} foreign record failed: {}", across_modules.stderr);
        let imported_c = directory.join(format!("{label}.c"));
        std::fs::write(&imported_c, across_modules.stdout).expect("imported C is writable");
        let imported_exe = directory.join(if cfg!(windows) { format!("{label}.exe") } else { label.to_string() });
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[imported_c, fixture.clone(), runtime_source.clone()],
            include_dirs: &[runtime.join("include")],
            output: imported_exe.clone(),
            profile: Profile::Debug,
            obj_dir: directory.join("obj"),
        }).expect("imported foreign record links to the C fixture");
        let imported_run = ember_build::output_started(&mut Command::new(imported_exe)).expect("imported record program runs");
        assert!(imported_run.status.success());
        assert_eq!(String::from_utf8_lossy(&imported_run.stdout).replace("\r\n", "\n"), "5\n23\n32\n34\n21\n23\n");
    }
    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignPair:\n    pub(read) left: i32\n    pub right: i32\nunsafe extern \"C\":\n    pub static mut shared_pair: ForeignPair\n")
        .expect("read-only foreign record module is writable");
    std::fs::write(&entry,
        "import imported\nfn main():\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        imported.shared_pair.left = 20\n")
        .expect("read-only foreign record entry is writable");
    let entry_arg = entry.to_string_lossy().into_owned();
    let readonly = ember(&["check", &entry_arg], &root);
    assert_ne!(readonly.exit, 0, "a pub(read) field was writable from another module");
    assert!(readonly.stderr.contains("E1050"), "expected read-only field error: {}", readonly.stderr);

    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignInner:\n    pub(read) value: i32\n@derive(Copy)\npub struct ForeignOuter:\n    pub inner: ForeignInner\nunsafe extern \"C\":\n    pub static mut nested_pair: ForeignOuter\n")
        .expect("nested read-only foreign record module is writable");
    std::fs::write(&entry,
        "import imported\nfn main():\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        imported.nested_pair.inner.value = 20\n")
        .expect("nested read-only foreign record entry is writable");
    let nested_readonly = ember(&["check", &entry_arg], &root);
    assert_ne!(nested_readonly.exit, 0, "a nested pub(read) field was writable from another module");
    assert!(nested_readonly.stderr.contains("E1050"), "expected nested read-only field error: {}", nested_readonly.stderr);
}

/// `[FFI-10]`, `[STA-1]` — a foreign static is a place in C's storage: a
/// record that is not `Copy` read and written field by field and assigned
/// whole, an element of an array in a record and of an array static written in
/// place (with `[EXP-2]`'s operand first), an immutable array, and a C function
/// pointer global. C reads back every write.
#[test]
fn foreign_statics_are_places_in_c_storage() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-10/accept_foreign_statics_as_places.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "foreign static C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains(" account;"), "the record global is declared: {}", emitted.stdout);

    let directory = temporary_directory("foreign-static-places");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_statics.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "foreign_places.exe" } else { "foreign_places" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture, runtime_source],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("foreign static places and C fixture link together");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "foreign static places program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"),
        "3\n253\n1009\n1001\n54\n66\n28\n23\n21\n13\n");
}

/// Build `source` (an Ember file) against `tests/fixtures/<fixture>` and the runtime in
/// `profile`, returning the program's path.
fn link_with_fixture(source: &Path, fixture: &str, profile: Profile, directory: &Path, label: &str) -> PathBuf {
    let root = workspace_root();
    let source_arg = source.to_string_lossy().into_owned();
    let profile_name = match profile { Profile::Debug => "debug", Profile::Development => "development", Profile::Release => "release" };
    let emitted = ember(&["build", &source_arg, "--emit", "c", "--profile", profile_name], &root);
    assert_eq!(emitted.exit, 0, "{label}: C emission failed: {}", emitted.stderr);
    let generated = directory.join(format!("{label}.c"));
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(fixture);
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { format!("{label}.exe") } else { label.to_string() });
    let objects = directory.join(format!("obj_{label}"));
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture, runtime_source],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile,
        obj_dir: objects,
    }).unwrap_or_else(|error| panic!("{label}: the program and its C fixture link: {error:?}"));
    output
}

/// `[FFI-36a]`, `[FFI-36b]`, `[FFI-33b]` — `adopt` makes a `ForeignBox` that frees its object
/// once with the destructor its type's declaration names (a status-returning one too); adopting a
/// live object a second time panics in every profile, naming both places; and a box of a
/// `threads(creator)` type that ends on another thread panics in the debug profile.
#[test]
fn adopted_foreign_objects_are_freed_by_their_destructor() {
    let root = workspace_root();
    let directory = temporary_directory("foreign-objects");
    let accepted = root.join(format!("tests/conformance/FFI-36a/accept_adopt_frees_with_the_declared_destructor.{SOURCE_EXT}"));
    for profile in [Profile::Debug, Profile::Development] {
        let program = link_with_fixture(&accepted, "foreign_objects.c", profile, &directory, &format!("adopt_{profile:?}"));
        let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
        assert!(run.status.success(), "{profile:?}: {}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "3 1\n4 3\n4 2\n3\n7 1\n507 0\n", "{profile:?}");
    }
    // `[FFI-11]` — `returns_owned` results (one may be null) and `owned` parameters.
    let transferred = root.join(format!("tests/conformance/FFI-36a/accept_returns_owned_and_owned_parameters.{SOURCE_EXT}"));
    for profile in [Profile::Debug, Profile::Development] {
        let program = link_with_fixture(&transferred, "foreign_objects.c", profile, &directory, &format!("owned_{profile:?}"));
        let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
        assert!(run.status.success(), "{profile:?}: {}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "2 1\nnone\n3 2\n3 1\n23 0\n1063 0\n1\n", "{profile:?}");
    }

    let declarations = "from std.ffi import adopt, ForeignBox\nimport std.thread\n\nunsafe extern \"C\":\n    @ffi(destroys(widget_free))\n    type Widget\n    @ffi(destroys(gadget_close), threads(creator))\n    type Gadget\n    fn widget_new(id: i32) -> *mut Widget\n    fn widget_free(widget: *mut Widget)\n    fn gadget_new(id: i32) -> *mut Gadget\n    fn gadget_close(gadget: *mut Gadget) -> i32\n\n";
    // `[FFI-36b]` — the second `adopt` of one live object, at line 23; the first is at 22.
    let twice = root.join(format!("tests/conformance/FFI-36b/accept_a_second_owner_panics_naming_both_places.{SOURCE_EXT}"));
    for profile in [Profile::Debug, Profile::Development] {
        let program = link_with_fixture(&twice, "foreign_objects.c", profile, &directory, &format!("twice_{profile:?}"));
        let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert!(!run.status.success(), "{profile:?}: adopting twice did not panic");
        let second_at = stderr.find("is adopted at").expect("the second place is named");
        let first_at = stderr.find("Ember already owns it since").expect("the first place is named");
        assert!(stderr[second_at..first_at].contains(&format!("both_places.{SOURCE_EXT}:23:")), "{profile:?}: {stderr}");
        assert!(stderr[first_at..].contains(&format!("both_places.{SOURCE_EXT}:22:")), "{profile:?}: {stderr}");
        assert!(!String::from_utf8_lossy(&run.stdout).contains("unreachable"));
    }

    let elsewhere = directory.join(format!("elsewhere.{SOURCE_EXT}"));
    std::fs::write(&elsewhere, format!("{declarations}fn make() -> ForeignBox[Gadget]:\n    unsafe:  # SAFETY(ffi): a fresh gadget\n        return adopt(gadget_new(2))\n\nfn main():\n    gadget = make()\n    worker = thread.spawn(owned fn() => gadget.as_ptr() == gadget.as_ptr())\n    println(worker.join())\n")).expect("program is writable");
    let program = link_with_fixture(&elsewhere, "foreign_objects.c", Profile::Debug, &directory, "elsewhere_debug");
    let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(!run.status.success(), "a creator box ended on another thread without a panic");
    assert!(stderr.contains("made on thread 1 (main) ends on thread") && stderr.contains("`threads(creator)`"), "{stderr}");
    let program = link_with_fixture(&elsewhere, "foreign_objects.c", Profile::Development, &directory, "elsewhere_development");
    let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
    assert!(run.status.success(), "development does not check the thread: {}", String::from_utf8_lossy(&run.stderr));
}

/// `[FFI-23]`, `[FFI-11]` — a `Retained` token lets C keep an Ember object's address: a class
/// object outlives the handle it was pinned from until the token is dropped, its address is the
/// one C keeps, and a pinned `Box`'s value is read by C through the kept address.
#[test]
fn retained_tokens_let_c_keep_ember_objects() {
    let root = workspace_root();
    let directory = temporary_directory("retained");
    let source = root.join(format!("tests/conformance/FFI-23/accept_retained_lets_foreign_code_keep_ember_memory.{SOURCE_EXT}"));
    for profile in [Profile::Debug, Profile::Development] {
        let program = link_with_fixture(&source, "foreign_objects.c", profile, &directory, &format!("retained_{profile:?}"));
        let run = ember_build::output_started(&mut Command::new(program)).expect("the program runs");
        assert!(run.status.success(), "{profile:?}: {}", String::from_utf8_lossy(&run.stderr));
        assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "still alive\n1\ndropped kept\nreleased\n3\n7\n", "{profile:?}");
    }
}

/// `[FFI-10]`, `[FFI-9]` — a function of an `extern "vectorcall"` block is
/// declared and called with that convention: the C compiler takes the
/// prototype (MSVC and Clang have it; GCC is told it has not).
#[test]
fn vectorcall_declarations_compile_with_the_convention() {
    let root = workspace_root();
    let directory = temporary_directory("vectorcall");
    let source = directory.join(ember_branding::source_file("vectorcall"));
    std::fs::write(&source, "unsafe extern \"vectorcall\":\n    safe fn scale_vector(x: f64, factor: f64) -> f64\n\nfn main():\n    println(scale_vector(2.0, 3.0))\n")
        .expect("program is writable");
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "vectorcall C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains("extern double EMBER_CC_VECTORCALL scale_vector(double"), "{}", emitted.stdout);
    let generated = directory.join("program.c");
    std::fs::write(&generated, &emitted.stdout).expect("generated C is writable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let object = directory.join(if cfg!(windows) { "program.obj" } else { "program.o" });
    let compiled = ember_build::compile_object(&toolchain, &generated, &[runtime.join("include")], &object, Profile::Debug);
    if matches!(toolchain, Toolchain::Gcc(_)) {
        let error = compiled.expect_err("GCC has no vectorcall convention");
        assert!(format!("{error:?}").contains("needs x86 and MSVC or Clang"), "{error:?}");
    } else {
        compiled.expect("the vectorcall prototype and call compile");
    }
}

/// `[FFI-9]`/`[FFI-10]` — C receives and returns a plain record by value.
#[test]
fn foreign_record_functions_exchange_values_with_c() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-10/accept_foreign_record_function_values.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "foreign record C emission failed: {}", emitted.stderr);

    let directory = temporary_directory("foreign-record-function");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_statics.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "foreign_record_function.exe" } else { "foreign_record_function" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture.clone(), runtime_source.clone()],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("foreign record function links to the C fixture");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "foreign record program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "8\n10\n17\n3\n4\n3\n13\n10\n16\n31\n22\n13\n10\n");

    let imported = directory.join(ember_branding::source_file("imported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "pub struct ForeignPair:\n    pub left: i32\n    pub right: i32\nunsafe extern \"C\":\n    pub safe fn mutate_pair_copy(pair: ForeignPair) -> i32\n    @ffi(link_name=\"foreign_pair_weighted\")\n    pub fn weighted_pair(pair: ForeignPair, weight: i32) -> i32\n")
        .expect("foreign record module is writable");
    for (label, source) in [
        ("from_import", "from imported import ForeignPair, mutate_pair_copy, weighted_pair\nfn main():\n    pair = ForeignPair(left = 3, right = 4)\n    println(mutate_pair_copy(pair))\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        println(weighted_pair(pair, 2))\n"),
        ("qualified", "import imported\nfn main():\n    pair = imported.ForeignPair(left = 3, right = 4)\n    println(imported.mutate_pair_copy(pair))\n    unsafe:  # SAFETY(ffi): each foreign item is used as its C declaration states\n        println(imported.weighted_pair(pair, 2))\n"),
    ] {
        std::fs::write(&entry, source).expect("foreign record entry is writable");
        let entry_arg = entry.to_string_lossy().into_owned();
        let emitted = ember(&["build", &entry_arg, "--emit", "c"], &root);
        assert_eq!(emitted.exit, 0, "{label} foreign record call failed: {}", emitted.stderr);
        let generated = directory.join(format!("{label}.c"));
        std::fs::write(&generated, emitted.stdout).expect("imported C is writable");
        let executable = directory.join(if cfg!(windows) { format!("{label}.exe") } else { label.to_string() });
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[generated, fixture.clone(), runtime_source.clone()],
            include_dirs: &[runtime.join("include")],
            output: executable.clone(),
            profile: Profile::Debug,
            obj_dir: directory.join("obj"),
        }).expect("imported foreign record function links to C");
        let run = ember_build::output_started(&mut Command::new(executable)).expect("imported record program runs");
        assert!(run.status.success(), "{label} foreign record program failed: {}",
            String::from_utf8_lossy(&run.stderr));
        assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "17\n10\n");
    }
}

/// `[FFI-9]` — C calls an exported Ember function with a record value.
#[test]
fn borrowed_record_export_receives_c_value() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-9/accept_borrowed_aggregate_export_by_value.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "borrowed record export C emission failed: {}", emitted.stderr);

    let directory = temporary_directory("borrowed-record-export");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let host = directory.join("host_program.c");
    let attach_symbol = format!("{}_rt_thread_attach", ember_branding::SYMBOL_PREFIX);
    std::fs::write(&host,
        format!("static int attach_calls = 0;\nvoid test_attach(void) {{ ++attach_calls; }}\n#define {attach_symbol} test_attach\n#include \"program.c\"\n#undef {attach_symbol}\nint32_t exported_attach_calls(void) {{ return attach_calls; }}\n"))
        .expect("instrumented C host is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_record_exports.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "borrowed_record_export.exe" } else { "borrowed_record_export" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[host, fixture, runtime_source],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("borrowed record export links to C");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "borrowed record export failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "10\n3\n21\n10\n13\n811\n10\n19\n15\n7\n8\n");
}

/// `[FFI-25]`, `[FFI-4]` — a C caller cannot observe an Ember panic as a return or unwind: the
/// panic aborts the process inside the Ember function, so it never crosses into the C code.
#[test]
fn exported_panic_aborts_inside_c_callback() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-25/accept_exported_panic_aborts.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "exported panic C emission failed: {}", emitted.stderr);

    let directory = temporary_directory("exported-panic-aborts");
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_export_panic.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "exported_panic.exe" } else { "exported_panic" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture, runtime_source],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("exported panic program links to C");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(!run.status.success(), "exported panic returned through C");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("before host call"), "C call was not reached: {stdout}");
    assert!(!stdout.contains("after host call"), "C returned after an Ember panic: {stdout}");
    assert!(String::from_utf8_lossy(&run.stderr).contains("panic inside an exported function"),
        "wrong failure: {}", String::from_utf8_lossy(&run.stderr));
    if cfg!(windows) {
        assert_eq!(run.status.code(), Some(3), "panic did not abort: {:?}", run.status);
    }
    #[cfg(unix)] {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(run.status.signal(), Some(6), "panic did not abort: {:?}", run.status);
    }
}

/// `[FFI-22]` — attaching is idempotent and local to the calling C thread.
#[test]
fn runtime_thread_attachment_is_thread_local() {
    let root = workspace_root();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/runtime_thread_attach.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let directory = temporary_directory("runtime-thread-attach");
    let output = directory.join(if cfg!(windows) { "runtime_thread_attach.exe" } else { "runtime_thread_attach" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[fixture],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("thread attachment host links");
    let run = ember_build::output_started(&mut Command::new(output)).expect("thread attachment host runs");
    assert!(run.status.success(), "thread attachment failed: {:?}\n{}",
        run.status, String::from_utf8_lossy(&run.stderr));
}

/// `[HASH-2]` — a `RandomState` is keyed once per process from the operating
/// system: one program run twice hashes one key two ways, and within a run
/// one way.
#[test]
fn random_state_is_keyed_per_process() {
    let root = workspace_root();
    let directory = temporary_directory("random-state");
    let program = directory.join(ember_branding::source_file("seeded"));
    std::fs::write(&program,
        "from std.collections import RandomState

fn seeded(s: str) -> u64:
    h = RandomState.new()
    s.hash(h)
    return h.finish()

fn main():
    println(seeded(\"key\") == seeded(\"key\"), seeded(\"key\"))
")
        .expect("the program is writable");
    let program = program.to_string_lossy().into_owned();
    let out = directory.join("target").to_string_lossy().into_owned();
    let first = ember(&["run", &program, "--out-dir", &out], &root);
    let second = ember(&["run", &program, "--out-dir", &out], &root);
    assert_eq!(first.exit, 0, "the first run failed: {}", first.stderr);
    assert_eq!(second.exit, 0, "the second run failed: {}", second.stderr);
    assert!(first.stdout.starts_with("true "), "one key hashed two ways in one run: {}", first.stdout);
    assert_ne!(first.stdout, second.stdout, "two runs drew one key");
}

/// `[FFI-26]` — an imported Ember module preserves its asserted linker name.
#[test]
fn imported_module_export_keeps_its_c_symbol() {
    let root = workspace_root();
    let directory = temporary_directory("imported-module-export");
    let imported = directory.join(ember_branding::source_file("exported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "@export(\"module_score\")\npub fn score(value: i32) -> i32:\n    return value + 1\n")
        .expect("exported module is writable");
    std::fs::write(&entry,
        "import exported\nunsafe extern \"C\":\n    safe fn call_module_score(value: i32) -> i32\nfn main():\n    println(call_module_score(41))\n")
        .expect("entry is writable");
    let entry_arg = entry.to_string_lossy().into_owned();
    let emitted = ember(&["build", &entry_arg, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "module export C emission failed: {}", emitted.stderr);
    let generated = directory.join("program.c");
    std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_module_export.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let output = directory.join(if cfg!(windows) { "module_export.exe" } else { "module_export" });
    let objects = directory.join("obj");
    std::fs::create_dir_all(&objects).expect("object directory is creatable");
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    ember_build::compile_and_link(&toolchain, &LinkRequest {
        sources: &[generated, fixture, runtime_source],
        include_dirs: &[runtime.join("include")],
        output: output.clone(),
        profile: Profile::Debug,
        obj_dir: objects,
    }).expect("imported Ember export links to C");
    let run = ember_build::output_started(&mut Command::new(output)).expect("linked program runs");
    assert!(run.status.success(), "module export failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "42");
}

/// `[FFI-33c]` — real C threads enforce explicit and imported-file policies
/// in every profile, including after shutdown and initialization elsewhere.
#[test]
fn exported_thread_contracts_hold_in_every_profile() {
    let root = workspace_root();
    let directory = temporary_directory("export-thread-contracts");
    let imported = directory.join(ember_branding::source_file("thread_defaults"));
    let creator_imported = directory.join(ember_branding::source_file("creator_defaults"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "#! language \"0.9.10\"\n#! threads main\n@export(\"default_score\")\npub fn default_value() -> i32:\n    return 43\n@export(\"override_score\", threads=any)\npub fn override_value() -> i32:\n    return 44\n")
        .expect("default policy module is writable");
    std::fs::write(&creator_imported,
        "#! language \"0.9.10\"\n#! threads creator\n@export(\"creator_default_score\")\npub fn default_value() -> i32:\n    return 46\n@export(\"creator_override_score\", threads=any)\npub fn override_value() -> i32:\n    return 47\n")
        .expect("creator default policy module is writable");
    std::fs::write(&entry,
        "import thread_defaults\nimport creator_defaults\n@export(\"main_score\", threads=main, on_panic=abort)\npub fn main_value() -> i32:\n    return 41\n@export(\"creator_score\", threads=creator)\npub fn creator_value() -> i32:\n    return 45\n@export(\"any_score\", threads=any)\npub fn any_value() -> i32:\n    return 42\n")
        .expect("explicit policy module is writable");
    let entry_arg = entry.to_string_lossy().into_owned();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_export_threads.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    for profile in [Profile::Debug, Profile::Development, Profile::Release] {
        let emitted = ember(&["build", &entry_arg, "--emit", "c", "--profile", profile.name()], &root);
        assert_eq!(emitted.exit, 0, "thread contract C emission failed: {}", emitted.stderr);
        let build = directory.join(profile.name());
        std::fs::create_dir_all(build.join("obj")).expect("object directory is creatable");
        let generated = build.join("program.c");
        std::fs::write(&generated, emitted.stdout).expect("generated C is writable");
        let output = build.join(if cfg!(windows) { "export_threads.exe" } else { "export_threads" });
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[generated, fixture.clone(), runtime_source.clone()],
            include_dirs: &[runtime.join("include")],
            output: output.clone(),
            profile,
            obj_dir: build.join("obj"),
        }).expect("thread policy exports link to C");
        for mode in [
            "allowed",
            "wrong",
            "creator_wrong",
            "default_wrong",
            "creator_default_wrong",
            "reinit",
            "reinit_creator",
            "uninitialized",
            "uninitialized_creator",
            "after_shutdown_creator",
        ] {
            let run = ember_build::output_started(Command::new(&output).arg(mode)).expect("C thread host runs");
            if mode == "allowed" {
                assert!(run.status.success(), "{} {mode}: {:?} {}", profile.name(), run.status,
                    String::from_utf8_lossy(&run.stderr));
            } else {
                assert!(!run.status.success(), "{} {mode}: prohibited export returned", profile.name());
                assert!(String::from_utf8_lossy(&run.stderr).contains("export requires the module initialization thread"),
                    "{} {mode}: wrong failure: {}", profile.name(), String::from_utf8_lossy(&run.stderr));
                if cfg!(windows) { assert_eq!(run.status.code(), Some(3)); }
                #[cfg(unix)] {
                    use std::os::unix::process::ExitStatusExt;
                    assert_eq!(run.status.signal(), Some(6));
                }
            }
        }
    }
}

#[test]
fn export_headers_link_from_separate_c_and_cpp_translation_units() {
    let root = workspace_root();
    let package = temporary_directory("export-header-api");
    let source = package.join(ember_branding::source_file("api"));
    std::fs::write(
        package.join(MANIFEST),
        "[package]\nname = \"header_api\"\nkind = \"lib\"\n",
    )
    .expect("header package manifest is writable");
    let source_fixture = root
        .join("tests")
        .join("conformance")
        .join("FFI-21")
        .join(ember_branding::source_file("accept_export_header_api"));
    std::fs::write(&source, std::fs::read_to_string(source_fixture).expect("header API fixture is readable"))
    .expect("header API source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let header_only_out = package.join("header-only");
    let header_only_out_arg = header_only_out.to_string_lossy().into_owned();
    let header_only = ember_with_env(
        &[
            "build",
            &source_arg,
            "--emit",
            "header",
            "--out-dir",
            &header_only_out_arg,
        ],
        &root,
        &ember_branding::cc_var(),
        "ember-compiler-that-does-not-exist",
    );
    assert_eq!(
        header_only.exit, 0,
        "header-only emission required a C compiler: {}",
        header_only.stderr
    );
    let header_only_path = header_only_out.join("debug/lib/header_api.h");
    assert!(header_only_path.is_file(), "header-only output was not written");
    assert!(
        !header_only_out.join("debug/c/api.c").exists(),
        "header-only emission also wrote a C translation unit"
    );
    let header = std::fs::read_to_string(&header_only_path).expect("header is readable");
    let leaf_type = ember_branding::mangled("HeaderLeaf");
    let envelope_type = ember_branding::mangled("HeaderEnvelope");
    let runtime_header = ember_branding::runtime_header();
    for declaration in [
        "header_score",
        "header_handle_roundtrip",
        "header_i128_identity",
        "header_creator",
        "header_any",
        "header_main",
        leaf_type.as_str(),
        envelope_type.as_str(),
        "struct HeaderHandle;",
        "threads: creator",
        "threads: any",
        "threads: main",
        "__cplusplus",
        "extern \"C\"",
        runtime_header.as_str(),
    ] {
        assert!(header.contains(declaration), "header omitted {declaration}:\n{header}");
    }
    let private_descriptor = ember_branding::runtime("native_fn_descriptor");
    let private_string = ember_branding::runtime("str");
    for private_name in [
        private_descriptor.as_str(),
        "private_callback_factory",
        "private_str_length",
        private_string.as_str(),
    ] {
        assert!(!header.contains(private_name), "header leaked {private_name}:\n{header}");
    }

    let combined_out = package.join("combined");
    let combined_out_arg = combined_out.to_string_lossy().into_owned();
    let combined = ember_with_env(
        &[
            "build",
            &source_arg,
            "--emit",
            "c",
            "--emit-header",
            "--out-dir",
            &combined_out_arg,
        ],
        &root,
        &ember_branding::cc_var(),
        "ember-compiler-that-does-not-exist",
    );
    assert_eq!(combined.exit, 0, "combined C/header emission failed: {}", combined.stderr);
    assert!(combined.stdout.contains("header_score"), "combined emission lost C stdout");
    let generated = combined_out.join("debug/c/api.c");
    let header_path = combined_out.join("debug/lib/header_api.h");
    assert!(generated.is_file(), "combined C output was not written");
    assert!(header_path.is_file(), "combined header output was not written");
    assert_eq!(
        std::fs::read_to_string(&header_path).expect("combined header is readable"),
        header,
        "header-only and combined emission disagree"
    );

    let c_host = package.join("host.c");
    let c_host_source = format!(
        "#include \"header_api.h\"\n#include \"{runtime_subdir}/{runtime_header}\"\n#include <stdint.h>\n\nstatic int32_t plus_one(int32_t value) {{ return value + 1; }}\nint main(void) {{\n    {rt_init}(NULL);\n    {leaf_type} leaf = {{ 19 }};\n    {envelope_type} envelope = {{ leaf, 22 }};\n    if (header_score(envelope, plus_one) != 42) return 1;\n    if (header_score(envelope, NULL) != 0) return 2;\n    if (header_creator() != 46 || header_any() != 47 || header_main() != 48) return 3;\n    const struct HeaderHandle *handle = (const struct HeaderHandle *)(uintptr_t)0x1234;\n    if (header_handle_roundtrip(handle) != handle) return 4;\n    {i128} wide = {i128_make}(0, 42);\n    if (!{i128_eq}(header_i128_identity(wide), wide)) return 5;\n    {rt_shutdown}();\n    return 0;\n}}\n",
        runtime_subdir = format!("{}_runtime", ember_branding::SYMBOL_PREFIX),
        runtime_header = runtime_header,
        rt_init = ember_branding::runtime("rt_init"),
        rt_shutdown = ember_branding::runtime("rt_shutdown"),
        i128 = ember_branding::runtime("i128"),
        i128_make = ember_branding::runtime("i128_make"),
        i128_eq = ember_branding::runtime("i128_eq"),
    );
    std::fs::write(&c_host, c_host_source)
    .expect("C header host is writable");
    let cpp_host = package.join("host.cpp");
    let cpp_host_source = format!(
        "#include \"header_api.h\"\n#include \"{runtime_subdir}/{runtime_header}\"\nint main() {{ {rt_init}(nullptr); int result = header_main(); {rt_shutdown}(); return result == 48 ? 0 : 1; }}\n",
        runtime_subdir = format!("{}_runtime", ember_branding::SYMBOL_PREFIX),
        runtime_header = runtime_header,
        rt_init = ember_branding::runtime("rt_init"),
        rt_shutdown = ember_branding::runtime("rt_shutdown"),
    );
    std::fs::write(&cpp_host, cpp_host_source)
    .expect("C++ header host is writable");

    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let runtime_source = runtime.join(format!("src/{}_rt.c", ember_branding::SYMBOL_PREFIX));
    let include_dirs = [runtime.join("include"), combined_out.join("debug/lib")];
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");

    let c_output = package.join(if cfg!(windows) { "header_c.exe" } else { "header_c" });
    let c_objects = package.join("c-objects");
    std::fs::create_dir_all(&c_objects).expect("C header object directory is creatable");
    ember_build::compile_and_link(
        &toolchain,
        &LinkRequest {
            sources: &[c_host.clone(), generated.clone(), runtime_source.clone()],
            include_dirs: &include_dirs,
            output: c_output.clone(),
            profile: Profile::Debug,
            obj_dir: c_objects,
        },
    )
    .expect("separate C host links against the emitted header");
    let c_run = ember_build::output_started(&mut Command::new(&c_output)).expect("C header host runs");
    assert!(c_run.status.success(), "C header host failed: {}", String::from_utf8_lossy(&c_run.stderr));

    let cpp_object = package.join(if cfg!(windows) { "host.obj" } else { "host.o" });
    compile_cpp_object(&toolchain, &cpp_host, &cpp_object, &include_dirs)
        .expect("C++ header host compiles");
    let cpp_output = package.join(if cfg!(windows) { "header_cpp.exe" } else { "header_cpp" });
    let cpp_objects = package.join("cpp-objects");
    std::fs::create_dir_all(&cpp_objects).expect("C++ header object directory is creatable");
    ember_build::compile_and_link(
        &toolchain,
        &LinkRequest {
            sources: &[generated, runtime_source, cpp_object],
            include_dirs: &include_dirs,
            output: cpp_output.clone(),
            profile: Profile::Debug,
            obj_dir: cpp_objects,
        },
    )
    .expect("C++ host links against the C exports");
    let cpp_run = ember_build::output_started(&mut Command::new(&cpp_output)).expect("C++ header host runs");
    assert!(cpp_run.status.success(), "C++ header host failed: {}", String::from_utf8_lossy(&cpp_run.stderr));
}

#[test]
fn export_header_rejects_reachable_native_only_types_without_writing_a_header() {
    let root = workspace_root();
    let package = temporary_directory("export-header-native-pointer");
    let source = package.join(ember_branding::source_file("native_pointer"));
    std::fs::write(
        package.join(MANIFEST),
        "[package]\nname = \"native_pointer_header\"\nkind = \"lib\"\n",
    )
    .expect("negative header package manifest is writable");
    let source_fixture = root
        .join("tests")
        .join("conformance")
        .join("FFI-21")
        .join(ember_branding::source_file("accept_header_native_callable_input"));
    std::fs::write(&source, std::fs::read_to_string(source_fixture).expect("negative header fixture is readable"))
    .expect("negative header source is writable");
    let source_arg = source.to_string_lossy().into_owned();

    let checked = ember(&["check", &source_arg], &root);
    assert_eq!(checked.exit, 0, "ordinary source checking changed: {}", checked.stderr);

    let out_dir = package.join("out");
    let out_arg = out_dir.to_string_lossy().into_owned();
    let emitted = ember(&["build", &source_arg, "--emit", "header", "--out-dir", &out_arg], &root);
    let diagnostics = format!("{}{}", emitted.stdout, emitted.stderr);
    assert_ne!(emitted.exit, 0, "native function pointers emitted a public header");
    assert!(
        diagnostics.contains("cannot emit C export header: reachable type")
            && diagnostics.contains("has no supported public C declaration"),
        "header rejection was not specific:\n{diagnostics}"
    );
    assert!(
        !out_dir.join("debug/lib/native_pointer_header.h").exists(),
        "failed header emission left an invalid header behind"
    );
}

#[test]
fn export_header_guards_distinguish_safe_names_and_unicode_paths() {
    let root = workspace_root();
    let source_text = "@export(\"score\")\npub fn score() -> i32:\n    return 42\n";
    let mut guards = Vec::new();
    for (label, package_name, source_name) in [
        ("dash", "foo-bar", "api"),
        ("underscore", "foo_bar", "api"),
        ("unicode", "café.api", "π.api"),
    ] {
        let package = temporary_directory(&format!("header-name-{label}"));
        std::fs::write(
            package.join(MANIFEST),
            format!("[package]\nname = \"{package_name}\"\nkind = \"lib\"\n"),
        )
        .expect("safe-name manifest is writable");
        let source = package.join(ember_branding::source_file(source_name));
        std::fs::write(&source, source_text).expect("safe-name source is writable");
        let source_arg = source.to_string_lossy().into_owned();
        let out = package.join("out");
        let out_arg = out.to_string_lossy().into_owned();
        let emitted = ember_with_env(
            &["build", &source_arg, "--emit", "header", "--out-dir", &out_arg],
            &root,
            &ember_branding::cc_var(),
            "ember-compiler-that-does-not-exist",
        );
        assert_eq!(emitted.exit, 0, "header name {package_name} failed: {}", emitted.stderr);
        let header_path = out.join("debug/lib").join(format!("{package_name}.h"));
        assert!(header_path.is_file(), "header path was not preserved: {}", header_path.display());
        let header = std::fs::read_to_string(&header_path).expect("safe-name header is readable");
        let guard = header.lines().find_map(|line| {
            line.strip_prefix("#ifndef ").map(str::trim).map(str::to_owned)
        }).expect("header guard exists");
        guards.push((package_name, guard));
    }
    assert_ne!(guards[0].1, guards[1].1, "distinct package names collided in the header guard");
}

/// `[FN-6]`, `[FFI-9]`, `[FFI-22]` — native callable values have a C adapter,
/// while their native calls preserve the original borrowed argument.
#[test]
fn native_callback_adapters_preserve_abi_and_attach_host_threads() {
    let root = workspace_root();
    let source = root.join(format!("tests/conformance/FFI-21/accept_native_callback_adapters.{SOURCE_EXT}"));
    let source_arg = source.to_string_lossy().into_owned();
    let directory = temporary_directory("native-callback-adapters");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/foreign_native_callbacks.c");
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = Toolchain::detect(requested.as_deref()).expect("C compiler is available");
    for profile in [Profile::Debug, Profile::Development, Profile::Release] {
        let emitted = ember(&["build", &source_arg, "--emit", "c", "--profile", profile.name()], &root);
        assert_eq!(emitted.exit, 0, "callback adapter C emission failed: {}", emitted.stderr);
        let build = directory.join(profile.name());
        std::fs::create_dir_all(build.join("obj")).expect("object directory is creatable");
        std::fs::write(build.join("program.c"), emitted.stdout).expect("generated C is writable");
        let output = build.join(if cfg!(windows) { "callbacks.exe" } else { "callbacks" });
        ember_build::compile_and_link(&toolchain, &LinkRequest {
            sources: &[fixture.clone()],
            include_dirs: &[runtime.join("include"), build.clone()],
            output: output.clone(),
            profile,
            obj_dir: build.join("obj"),
        }).expect("native callback adapters link to C");
        let run = ember_build::output_started(&mut Command::new(output)).expect("native callbacks run on a host thread");
        assert!(run.status.success(), "{} callback host failed: {:?}\n{}", profile.name(),
            run.status, String::from_utf8_lossy(&run.stderr));
    }
}

/// `[MAN-3]` — a manifest must not silently accept configuration for a lint
/// that the compiler does not define. The manifest lives beside an otherwise
/// valid standalone source file so this exercises the driver's nearest-package
/// lookup rather than a parser helper in isolation.
#[test]
fn unknown_manifest_lint_is_rejected() {
    let root = workspace_root();
    let package = temporary_directory("unknown-manifest-lint");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[lints]\nnot_a_lint = \"warn\"\n",
    )
    .expect("manifest is writable");
    std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_ne!(
        checked.exit, 0,
        "unknown manifest lint unexpectedly compiled"
    );
    let rendered = without_source_echo(&checked.stderr);
    assert!(
        rendered.contains("error[E9010]"),
        "unknown lint did not use E9010:\n{}",
        checked.stderr
    );
    assert!(
        rendered.contains("unknown lint `not_a_lint`"),
        "unknown lint was not identified:\n{}",
        checked.stderr
    );
    assert!(
        checked.stderr.contains(MANIFEST),
        "manifest diagnostic did not name its source file:\n{}",
        checked.stderr
    );
}

/// `[EFF-19]` — `@realtime` stands for the set the package's manifest declares in `[realtime]
/// contracts`: with `["noalloc"]`, a `@realtime` function may panic but not allocate, and the error
/// names both `@realtime` and the contract; a name that is no contract is `E9001`.
#[test]
fn realtime_stands_for_the_manifests_contracts() {
    let root = workspace_root();
    let package = temporary_directory("realtime-contracts");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(package.join(MANIFEST), "[realtime]\ncontracts = [\"noalloc\"]\n").expect("manifest is writable");
    std::fs::write(
        &source,
        "@realtime\nfn stop(x: int) -> int:\n    if x < 0:\n        panic(\"negative\")\n    return x\n\n\
         @realtime\nfn grow(x: int) -> int:\n    xs = [x]\n    return xs.len()\n\n\
         fn main():\n    println(stop(1), grow(2))\n",
    )
    .expect("source is writable");
    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let rendered = without_source_echo(&checked.stderr);
    assert_ne!(checked.exit, 0, "an allocation under `@realtime` = `@noalloc` compiled");
    assert!(rendered.contains("error[E4001]") && rendered.contains("`@realtime` (`@noalloc`) function `grow`"), "{}", checked.stderr);
    assert!(rendered.contains("`@realtime` stands for `@noalloc` here"), "{}", checked.stderr);
    assert!(!rendered.contains("E4040"), "a panic is allowed when the set has no `nopanic(explicit)`:\n{}", checked.stderr);

    std::fs::write(package.join(MANIFEST), "[realtime]\ncontracts = [\"noalloc\", \"fast\"]\n").expect("manifest is writable");
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    let rendered = without_source_echo(&checked.stderr);
    assert!(rendered.contains("error[E9001]") && rendered.contains("`fast` is not a contract `@realtime` can stand for"), "{}", checked.stderr);
}

/// `[MAN-3]` — the registry spelling and the descriptive lint names used by
/// the manifest examples all remain valid configuration keys.
#[test]
fn known_manifest_lints_are_accepted() {
    let root = workspace_root();
    let package = temporary_directory("known-manifest-lints");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[lints]\nl3013 = \"warn\"\nl3014 = \"warn\"\nunused = \"warn\"\npotential_cycle = \"warn\"\nlarge_copy = { level = \"warn\", threshold = 256 }\n",
    )
    .expect("manifest is writable");
    std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(
        checked.exit, 0,
        "known manifest lints were rejected:\n{}",
        checked.stderr
    );
}

/// `[PRF-3]` and `[EXC-14]` — a manifest must not silently revive a removed
/// safety switch or accept a misspelled profile setting.
#[test]
fn removed_safety_switches_and_unknown_profile_keys_are_rejected() {
    let root = workspace_root();
    for (name, manifest, key) in [
        ("unchecked-profile", "[profiles.release]\nexclusivity = \"unchecked\"\n", "exclusivity"),
        ("unchecked-build", "[build]\nexclusivity = \"unchecked\"\n", "exclusivity"),
        ("overflow", "[profiles.release]\noverflow = \"unchecked\"\n", "overflow"),
        ("bounds", "[profiles.release]\nbounds_checks = false\n", "bounds_checks"),
        ("gpu-validation", "[gpu]\nvalidate = false\n", "gpu.validate"),
        ("gpu-validation-dotted", "[build]\ngpu.validate = false\n", "gpu.validate"),
        ("unknown-profile-key", "[profiles.debug]\nbakcend = \"c\"\n", "bakcend"),
    ] {
        let package = temporary_directory(name);
        let source = package.join(ember_branding::source_file("main"));
        std::fs::write(package.join(MANIFEST), manifest).expect("manifest is writable");
        std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");
        let checked = ember(&["check", &source.to_string_lossy()], &root);
        let _ = std::fs::remove_dir_all(&package);
        assert_ne!(checked.exit, 0, "{name} was accepted:\n{}", checked.stderr);
        assert!(checked.stderr.contains("error[E9001]"), "{name} did not emit E9001:\n{}", checked.stderr);
        assert!(checked.stderr.contains(key), "{name} did not name {key}:\n{}", checked.stderr);
        assert!(checked.stderr.contains(MANIFEST), "{name} did not point to the manifest:\n{}", checked.stderr);
    }
}

/// `[MAN-1]`, `[MAN-8]` — an unknown key in any manifest section is `E9001`,
/// naming the key and the keys that section accepts; an unknown section is
/// `E9001` naming the sections. D-621: `[comptime]`'s keys are `max_steps`
/// and `max_heap_mb`.
#[test]
fn unknown_manifest_keys_name_the_keys_their_section_accepts() {
    let root = workspace_root();
    for (name, manifest, message, help) in [
        ("package", "[package]\nname = \"p\"\nbogus = 1\n", "unknown key `bogus` in `[package]`",
            "`[package]` accepts `name`, `version`, `language`, `kind`, `layers`, `edition_lints`"),
        ("build", "[build]\nreloads = \"all\"\n", "unknown key `reloads` in `[build]`", "`reload`"),
        ("comptime", "[comptime]\nsteps = 5\n", "unknown key `steps` in `[comptime]`",
            "`[comptime]` accepts `max_steps`, `max_heap_mb`"),
        ("jobs", "[jobs]\nworker = 2\n", "unknown key `worker` in `[jobs]`", "`[jobs]` accepts `workers`"),
        ("profile", "[profiles.debug]\nbakcend = \"c\"\n", "unknown key `bakcend` in `[profiles.debug]`", "`inherits`"),
        ("cpp", "[cpp.engine]\ncompilr = \"clang\"\n", "unknown key `compilr` in `[cpp.engine]`", "`compiler`"),
        ("top-level", "name = \"p\"\n", "unknown key `name` in the manifest's top level", "`[package]`"),
        ("section", "[packages]\nname = \"p\"\n", "unknown manifest section `[packages]`", "`[package]`, `[dependencies]`"),
    ] {
        let package = temporary_directory(&format!("manifest-{name}"));
        let source = package.join(ember_branding::source_file("main"));
        std::fs::write(package.join(MANIFEST), manifest).expect("manifest is writable");
        std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");
        let checked = ember(&["check", &source.to_string_lossy()], &root);
        let _ = std::fs::remove_dir_all(&package);
        assert_ne!(checked.exit, 0, "[MAN-1] {name} was accepted:\n{}", checked.stderr);
        assert!(checked.stderr.contains("error[E9001]") && checked.stderr.contains(message), "[MAN-1] {name}:\n{}", checked.stderr);
        assert!(checked.stderr.contains(help), "[MAN-1] {name} did not name the accepted keys {help:?}:\n{}", checked.stderr);
        assert!(checked.stderr.contains(MANIFEST), "[MAN-1] {name} did not point to the manifest:\n{}", checked.stderr);
    }
    // Every key `[MAN-8]` names is accepted. The values are real ones (a dependency is resolved,
    // the language is the current one); a `{ git, rev }` dependency, which needs a repository, is
    // the package tests' (`a_git_dependency_is_checked_out_at_its_revision`).
    let package = temporary_directory("manifest-every-key");
    let source = package.join(ember_branding::source_file("main"));
    for dependency in ["geometry", "parser"] {
        let dir = package.join("deps").join(dependency);
        std::fs::create_dir_all(dir.join("src")).expect("the dependency is writable");
        std::fs::write(dir.join(MANIFEST), format!("[package]\nname = \"{dependency}\"\nversion = \"1.0.0\"\nkind = \"lib\"\nlayers = [\"core\"]\n"))
            .expect("the dependency's manifest is writable");
        std::fs::write(dir.join("src").join(ember_branding::source_file("lib")), "pub fn one() -> i32:\n    return 1\n")
            .expect("the dependency's source is writable");
    }
    std::fs::write(
        package.join(MANIFEST),
        format!("[package]\nname = \"p\"\nversion = \"0.1.0\"\nlanguage = \"{}\"\nkind = \"bin\"\nlayers = [\"core\"]\nedition_lints = \"strict\"\n\
         [dependencies]\ngeometry = {{ path = \"deps/geometry\" }}\n[dependencies.parser]\npath = \"deps/parser\"\n\
         [build]\ntarget = \"native\"\nbackend = \"c\"\nc_compiler = \"auto\"\nruntime = \"static\"\nmax_instantiations = 200\n\
         [profiles.development]\nlto = \"thin\"\n[c]\ncompiler = \"auto\"\nflags = []\n[link]\nlibs = []\n[jobs]\nworkers = \"auto\"\n\
         [realtime]\ncontracts = [\"noalloc\"]\n[comptime]\nmax_steps = 100000000\nmax_heap_mb = 256\n[lints]\nunused = \"warn\"\n\
         [cpp.engine]\ncompiler = \"clang\"\nstandard = \"c++20\"\ndefines = []\ninclude_paths = []\nflags = []\n[ffi]\nevidence = \".ember/ffi-evidence\"\n",
        ember_parser::LANGUAGE_VERSION),
    )
    .expect("manifest is writable");
    std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");
    let checked = ember(&["check", &source.to_string_lossy()], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "[MAN-8] the manifest's own keys were rejected:\n{}", checked.stderr);
}

/// D-621 — `[CT-3]`'s limits are `[comptime] max_steps` and `max_heap_mb`
/// (`[MAN-8]`): a lower step limit stops an evaluation the default allows.
#[test]
fn comptime_limits_come_from_max_steps_and_max_heap_mb() {
    let root = workspace_root();
    let package = temporary_directory("manifest-comptime-limits");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(package.join(MANIFEST), "[comptime]\nmax_steps = 1000\nmax_heap_mb = 64\n").expect("manifest is writable");
    std::fs::write(
        &source,
        "fn count() -> int:\n    n = 0\n    for i in 0..100000:\n        n += i % 3\n    return n\n\nconst N: int = count()\n\nfn main():\n    println(N)\n",
    )
    .expect("source is writable");
    let checked = ember(&["check", &source.to_string_lossy()], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_ne!(checked.exit, 0, "[CT-3] the step limit was not read:\n{}", checked.stderr);
    assert!(
        checked.stderr.contains("error[E6001]") && checked.stderr.contains("takes more than 1000 steps")
            && checked.stderr.contains("`max_steps = …` under `[comptime]`"),
        "[CT-3] {}",
        checked.stderr
    );
}

/// `[CLI-1]` — every command accepts `--json` and exits non-zero on error:
/// `fmt` (whether the file is formatted, and the layout it prints),
/// `explain` (a code's record and page), `explain --cycle`, `--version`.
#[test]
fn every_command_accepts_json() {
    let root = workspace_root();
    let parse = |run: &Run| -> serde_json::Value {
        serde_json::from_str(run.stdout.trim()).unwrap_or_else(|error| panic!("[CLI-1] not JSON ({error}):\n{}{}", run.stdout, run.stderr))
    };
    let package = temporary_directory("cli-json");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(&source, "fn main():\n    println(1)\n").expect("source is writable");
    let file = source.to_string_lossy().into_owned();
    let printed = ember(&["fmt", "--json", &file], &root);
    assert_eq!(printed.exit, 0, "{}", printed.stderr);
    let value = parse(&printed);
    assert_eq!(value["formatted"], true, "{}", printed.stdout);
    assert_eq!(value["text"], "fn main():\n    println(1)\n", "{}", printed.stdout);
    std::fs::write(&source, "fn main():\n    println(1)    \n").expect("source is writable");
    let check = ember(&["fmt", &file, "--check", "--json"], &root);
    assert_eq!(check.exit, 1, "[CLI-1] an unformatted file passed `--check`:\n{}", check.stdout);
    assert_eq!(parse(&check)["formatted"], false, "{}", check.stdout);
    std::fs::write(&source, "fn main(:\n").expect("source is writable");
    let broken = ember(&["fmt", &file, "--json"], &root);
    assert_eq!(broken.exit, 1, "{}", broken.stdout);
    assert!(broken.stdout.contains("\"code\":\"E0100\""), "[CLI-1] a parse error was not JSON:\n{}{}", broken.stdout, broken.stderr);
    let _ = std::fs::remove_dir_all(&package);

    let explained = ember(&["explain", "E9001", "--json"], &root);
    assert_eq!(explained.exit, 0, "{}", explained.stderr);
    let value = parse(&explained);
    assert_eq!(value["code"], "E9001");
    assert_eq!(value["rule"], "[MAN-1], [PRF-3]");
    let unknown = ember(&["explain", "--json", "E99999"], &root);
    assert_ne!(unknown.exit, 0, "[CLI-1] an unknown code succeeded");

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("cycle_inspection.{SOURCE_EXT}"));
    let fixture = fixture.to_string_lossy().into_owned();
    let cycle = ember(&["explain", "--cycle", &fixture, "Root.child", "--json"], &root);
    assert_eq!(cycle.exit, 0, "{}", cycle.stderr);
    let value = parse(&cycle);
    assert_eq!(value["target"], "Root.child");
    assert_eq!(value["static_paths"][0], "Root.child -> Child.leaf -> Leaf.root -> Root", "{}", cycle.stdout);

    let version = ember(&["--version", "--json"], &root);
    assert_eq!(parse(&version)["version"], env!("CARGO_PKG_VERSION"));
}

/// `[CLI-2]` — `--emit obj` compiles the C to object files and links
/// nothing: the program's unit, and a relaxed unit of its own for a
/// `@fastmath` function (`[CG-C-11]`).
#[test]
fn emit_obj_writes_objects_without_linking() {
    let root = workspace_root();
    let package = temporary_directory("emit-obj");
    let source = package.join(ember_branding::source_file("objects"));
    std::fs::write(&source, "@fastmath\nfn scale(x: f64) -> f64:\n    return x * 2.0\n\nfn main():\n    println(scale(1.5))\n")
        .expect("source is writable");
    let out = package.join("out");
    let built = ember(&["build", &source.to_string_lossy(), "--emit", "obj", "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(built.exit, 0, "[CLI-2] {}", built.stderr);
    let objects: Vec<String> = std::fs::read_dir(out.join("debug").join("obj"))
        .expect("the object directory exists")
        .map(|entry| entry.expect("an entry").file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".obj") || name.ends_with(".o"))
        .collect();
    assert!(objects.iter().any(|name| name.starts_with("objects.") && !name.contains("fast")), "[CLI-2] no program object: {objects:?}");
    assert!(objects.iter().any(|name| name.starts_with("objects.fast")), "[CLI-2] no relaxed object: {objects:?}");
    let linked = std::fs::read_dir(out.join("debug").join("bin")).map(|entries| entries.count()).unwrap_or(0);
    assert_eq!(linked, 0, "[CLI-2] `--emit obj` linked an executable");
    assert!(built.stderr.contains("built "), "{}", built.stderr);
    let _ = std::fs::remove_dir_all(&package);
}

/// `[PRF-3]` — the specified profile settings remain accepted in a custom
/// profile, including its required inheritance key.
#[test]
fn known_profile_keys_are_accepted() {
    let root = workspace_root();
    let package = temporary_directory("known-profile-keys");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[profiles.checked]\ninherits = \"debug\"\nopt = 1\ndebug_info = \"lines\"\nstrip = false\nbacktrace = true\ndebug_assert = true\nleak_report = true\nlock_order = true\nsanitizers = []\nlto = \"thin\"\n",
    )
    .expect("manifest is writable");
    std::fs::write(&source, "fn main():\n    pass\n").expect("source is writable");
    let checked = ember(&["check", &source.to_string_lossy()], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "valid profile settings were rejected:\n{}", checked.stderr);
}

/// `[EXC-7]` — the opt-in `L3013` warning must identify a long-term mutable
/// class access that remains live across a virtual call in the same open
/// hierarchy. The warning is advisory, so the otherwise-valid program still
/// checks successfully.
#[test]
fn enabled_l3013_warns_for_live_class_access_across_virtual_call() {
    let root = workspace_root();
    let package = temporary_directory("l3013-virtual-access");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[lints]\nl3013 = \"warn\"\n",
    )
    .expect("manifest is writable");
    std::fs::write(
        &source,
        "open class Base:\n    value: i32\n\n    fn init(mut self):\n        self.value = 7\n\n    virtual fn ping(self) -> i32:\n        return self.value\n\n    virtual fn relay(mut self) -> i32:\n        return self.ping()\n\nclass Derived(Base):\n    fn init(mut self):\n        super.init()\n\n    override fn ping(self) -> i32:\n        return self.value\n\nfn main():\n    derived = Derived()\n    base: Base = derived\n    println(base.relay())\n",
    )
    .expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "program did not check:\n{}", checked.stderr);
    let rendered = without_source_echo(&checked.stderr);
    assert!(
        rendered.contains("warning[L3013]"),
        "enabled L3013 was not emitted:\n{}",
        checked.stderr
    );
    assert!(
        rendered.contains("long-term access to `self` is live across this call"),
        "L3013 did not identify the live class access and call:\n{}",
        checked.stderr
    );
}

/// `[EXC-7]` — `L3013` is advisory and disabled until a package explicitly
/// elects to receive it. The identical re-entrant virtual call must therefore
/// remain quiet without an `[lints]` setting.
#[test]
fn l3013_is_disabled_without_a_manifest_setting() {
    let root = workspace_root();
    let package = temporary_directory("l3013-default-off");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        &source,
        "open class Base:\n    value: i32\n\n    fn init(mut self):\n        self.value = 7\n\n    virtual fn ping(self) -> i32:\n        return self.value\n\n    virtual fn relay(mut self) -> i32:\n        return self.ping()\n\nclass Derived(Base):\n    fn init(mut self):\n        super.init()\n\n    override fn ping(self) -> i32:\n        return self.value\n\nfn main():\n    derived = Derived()\n    base: Base = derived\n    println(base.relay())\n",
    )
    .expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "program did not check:\n{}", checked.stderr);
    assert!(
        !without_source_echo(&checked.stderr).contains("L3013"),
        "default-disabled L3013 was emitted:\n{}",
        checked.stderr
    );
}

/// `[EXC-7]` — a final class has no open hierarchy to re-enter. Its
/// `virtual` spelling remains covered by `W2111`, but enabling `L3013` must
/// not add a long-term-access warning for the statically closed call.
#[test]
fn l3013_ignores_virtual_calls_in_a_final_class() {
    let root = workspace_root();
    let package = temporary_directory("l3013-final-class");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[lints]\nl3013 = \"warn\"\n",
    )
    .expect("manifest is writable");
    std::fs::write(
        &source,
        "class FinalThing:\n    value: i32\n\n    fn init(mut self):\n        self.value = 7\n\n    virtual fn ping(self) -> i32:\n        return self.value\n\n    virtual fn relay(mut self) -> i32:\n        return self.ping()\n\nfn main():\n    thing = FinalThing()\n    println(thing.relay())\n",
    )
    .expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "program did not check:\n{}", checked.stderr);
    assert!(
        !without_source_echo(&checked.stderr).contains("L3013"),
        "L3013 warned for a final class:\n{}",
        checked.stderr
    );
}

/// `[EXC-7]` — erasing a final class behind `ref dyn` must not turn its
/// statically closed receiver hierarchy into a re-entry warning.
#[test]
fn l3013_ignores_dynamic_calls_from_a_final_class() {
    let root = workspace_root();
    let package = temporary_directory("l3013-final-dynamic-class");
    let source = package.join(ember_branding::source_file("main"));
    std::fs::write(
        package.join(MANIFEST),
        "[lints]\nl3013 = \"warn\"\n",
    )
    .expect("manifest is writable");
    std::fs::write(
        &source,
        "interface Probe:\n    fn read(self) -> i32\n\nclass FinalThing implements Probe:\n    value: i32\n\n    fn init(mut self):\n        self.value = 7\n\n    fn read(self) -> i32:\n        return self.value\n\n    fn relay(mut self) -> i32:\n        probe: ref dyn Probe = ref self\n        return probe.read()\n\nfn main():\n    thing = FinalThing()\n    println(thing.relay())\n",
    )
    .expect("source is writable");

    let source_arg = source.to_string_lossy().into_owned();
    let checked = ember(&["check", &source_arg], &root);
    let _ = std::fs::remove_dir_all(&package);
    assert_eq!(checked.exit, 0, "program did not check:\n{}", checked.stderr);
    assert!(
        !without_source_echo(&checked.stderr).contains("L3013"),
        "L3013 warned for dynamic dispatch from a final class:\n{}",
        checked.stderr
    );
}

#[test]
fn dynamic_access_safety_side_table_is_written() {
    let root = workspace_root();
    // A program whose one `mut self` call must stay checked (ODR-085 removes
    // the check from a program where nothing held could make it fail).
    let source = format!(
        "tests/run-pass/class_mut_method_access_kept.{SOURCE_EXT}"
    );
    let out_dir = temporary_directory("safety-side-table");
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
        .join("class_mut_method_access_kept.safety.json");
    let json = std::fs::read_to_string(&side_table)
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
    assert!(json.contains("\"schema\":1"), "missing side-table schema: {json}");
    assert!(json.contains("\"kind\":\"Aliasing\""), "missing access kind: {json}");
    // `[EXC-15]` — a `mut self` call's whole-object write is taken by its
    // caller, so the check belongs to `main`.
    assert!(json.contains("\"function\":\"main\""), "missing function identity: {json}");
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
            "main",
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
        function_filtered.stdout.contains("Function: main"),
        "function-filtered inspect omitted main:\n{}",
        function_filtered.stdout
    );
    let function_json = ember(
        &[
            "inspect",
            "--safety",
            "--json",
            "--function",
            "main",
            &side_table_arg,
        ],
        &root,
    );
    assert_eq!(function_json.exit, 0, "JSON function filter failed:\n{}", function_json.stderr);
    assert!(
        function_json.stdout.contains("\"function\":\"main\""),
        "JSON function filter omitted main:\n{}",
        function_json.stdout
    );
}

/// `[EFF-10]` — the effect set says only which kinds of check a function has; the safety side table
/// lists each one: every check left in with its kind, location, mechanism and reason, and every one
/// removed with the proof that removed it (the loop's index is in range, `[RNG-4]`). `[EFF-11]` —
/// every check left in carries one reason, one of the table's five.
#[test]
fn the_safety_table_lists_every_check_by_site() {
    let root = workspace_root();
    let dir = temporary_directory("checks-by-site");
    let source = dir.join(format!("sites.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "fn get(xs: Array[int], i: int) -> int:\n    return xs[i]\n\n\
         fn total(xs: Array[int]) -> int:\n    t = 0\n    for i in 0..xs.len():\n        t += xs[i]\n    return t\n\n\
         fn main():\n    xs = [1, 2, 3]\n    println(get(xs, 1) + total(xs))\n",
    )
    .expect("write the program");
    let out_dir = dir.join("out");
    let build = ember(&["build", &source.to_string_lossy(), "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(build.exit, 0, "build failed:\n{}", build.stderr);
    let table = out_dir.join("debug").join("inspect").join("sites.safety.json");
    let json = std::fs::read_to_string(&table).expect("the side table");
    let _ = std::fs::remove_dir_all(&dir);
    // By where it is written: `get` and `total` may be inlined into `main`.
    let entry = |at: &str, kind: &str, mechanism: &str, status: &str, reason: &str| {
        json.split("},{").any(|entry| {
            entry.contains(&format!("sites.{SOURCE_EXT}:{at}\""))
                && entry.contains(&format!("\"kind\":\"{kind}\""))
                && entry.contains(&format!("\"mechanism\":\"{mechanism}\""))
                && entry.contains(&format!("\"status\":\"{status}\""))
                && entry.contains(&format!("\"reason\":\"{reason}\""))
        })
    };
    assert!(entry("2:12", "Bounds", "index test", "emitted", "not_proven_by_analysis"), "{json}");
    assert!(entry("7:9", "Overflow", "overflow test", "emitted", "not_proven_by_analysis"), "{json}");
    assert!(entry("7:17", "Bounds", "range facts", "elided", "range_facts"), "{json}");
    let table: serde_json::Value = serde_json::from_str(&json).expect("the side table is JSON");
    let reasons = ["not_provable_in_principle", "not_proven_by_analysis", "requested_by_type", "inherent_to_mechanism", "establishes_static_fact"];
    for check in table["checks"].as_array().expect("the checks").iter().filter(|check| check["status"] == "emitted") {
        assert!(check["reason"].as_str().is_some_and(|reason| reasons.contains(&reason)), "[EFF-11] {check}");
    }
}

/// `[DSJ-5]` — each `assert_disjoint` comparison is a `RuntimeCheck(Aliasing)` whose reason is
/// `establishes_static_fact` in the safety side table, unless the compiler already knows the views
/// apart (views of two different lists here), when no comparison is emitted and the table says so.
#[test]
fn assert_disjoint_records_its_comparison_or_leaves_it_out() {
    let root = workspace_root();
    let dir = temporary_directory("disjoint-record");
    let source = dir.join(format!("disjoint.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "from std.mem import assert_disjoint_or_panic\n\n\
         fn pair(a: Span[int], b: Span[int]) -> int:\n    (p, q) = assert_disjoint_or_panic(a, b)\n    return p.len() + q.len()\n\n\
         fn main():\n    x: Array[int] = [1, 2, 3]\n    y: Array[int] = [4]\n\
         \x20   (p, q) = assert_disjoint_or_panic(x.as_span(), y.as_span())\n    println(p.len() + q.len(), pair(x[..], y[..]))\n",
    )
    .expect("write the program");
    let out_dir = dir.join("out");
    let build = ember(&["build", &source.to_string_lossy(), "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(build.exit, 0, "build failed:\n{}", build.stderr);
    let table = out_dir.join("debug").join("inspect").join("disjoint.safety.json");
    let json = std::fs::read_to_string(&table).expect("the side table");
    let _ = std::fs::remove_dir_all(&dir);
    // By where it is written: `pair` may be inlined into `main`.
    let entry = |line: u32, status: &str, reason: &str| {
        json.split("},{").any(|entry| {
            entry.contains(&format!("disjoint.{SOURCE_EXT}:{line}:"))
                && entry.contains("\"mechanism\":\"assert_disjoint\"")
                && entry.contains(&format!("\"status\":\"{status}\""))
                && entry.contains(&format!("\"reason\":\"{reason}\""))
        })
    };
    assert!(entry(4, "emitted", "establishes_static_fact"), "{json}");
    assert!(entry(10, "elided", "known_disjoint"), "{json}");
}

/// `[THR-14]` — `Atomic[i128]` and `Atomic[u128]` under clang as under MSVC (the conformance test's
/// own compiler): the 16-byte compare-exchange inline (`-mcx16`), nothing left to a library the
/// program does not link, and the same results in both profiles. A machine without clang skips it.
#[test]
fn atomics_of_128_bits_run_under_clang() {
    let root = workspace_root();
    let source = format!("tests/conformance/THR-14/accept_128_bit_atomics.{SOURCE_EXT}");
    for profile in ["debug", "development"] {
        let run = ember(&["run", &source, "--profile", profile, "--cc", "clang"], &root);
        if run.exit != 0 && run.stderr.contains("clang") && run.stderr.contains("not found") {
            return;
        }
        assert_eq!(run.exit, 0, "{profile}:\n{}\n{}", run.stdout, run.stderr);
        assert!(run.stdout.contains("18446744073709551615 18446744073709551616\n"), "{profile}: {}", run.stdout);
        assert!(run.stdout.ends_with("4000\n"), "{profile}: {}", run.stdout);
    }
}

/// `[UNS-8]` — `ember tcb` lists every `unsafe:` block and `unsafe fn` with its safety note and
/// category (or that it has none), and for a block the unsafe functions it calls with the
/// obligation each states (`@safety`, `[UNS-7]`).
#[test]
fn tcb_lists_every_unsafe_with_its_note_and_obligations() {
    let root = workspace_root();
    let dir = temporary_directory("tcb-report");
    let source = dir.join(format!("trusted.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "@safety(\"`n` is below 100\")\n\
         # SAFETY(performance): its callers bound `n`, so the product needs no check\n\
         pub unsafe fn scale(n: i64) -> i64:\n    return n * 2\n\n\
         fn main():\n    total = 0\n\
         \x20   unsafe:  # SAFETY: 3 is below 100\n\
         \x20       total += scale(3)\n\
         \x20   unsafe:\n        total += scale(4)\n    println(total)\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let report = ember(&["tcb", &source_arg], &root);
    assert_eq!(report.exit, 0, "tcb failed:\n{}\n{}", report.stdout, report.stderr);
    let out = &report.stdout;
    assert!(out.contains("Unsafe (3):"), "{out}");
    assert!(out.contains("unsafe fn `scale`  SAFETY(performance): its callers bound `n`, so the product needs no check"), "{out}");
    assert!(out.contains("block in `main`  SAFETY: 3 is below 100"), "{out}");
    assert!(out.contains("block in `main`  no safety note (W3012)"), "{out}");
    assert_eq!(out.matches("calls `scale`: `n` is below 100").count(), 2, "{out}");
    let json = ember(&["tcb", "--json", &source_arg], &root);
    assert_eq!(json.exit, 0, "tcb --json failed:\n{}", json.stderr);
    assert!(json.stdout.contains("\"category\":\"performance\""), "{}", json.stdout);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[FFI-10]`, `[TCB-1]`, `[TCB-2]`, `[TCB-3]` — `ember tcb` separates what the language guarantees
/// from what the program supplies, and lists, grouped by module, every fact its foreign
/// declarations assert (`safe fn` among them), each with its source and its grade: asserted, for
/// nothing a declaration writes raises one.
#[test]
fn tcb_lists_every_foreign_fact_with_its_source_and_grade() {
    let root = workspace_root();
    let dir = temporary_directory("tcb-foreign");
    let helper = dir.join(format!("clib.{SOURCE_EXT}"));
    std::fs::write(
        &helper,
        "unsafe extern \"C\":\n    @ffi(effects=[], deterministic)\n    pub safe fn abs(x: i32) -> i32\n    pub fn rand() -> i32\n",
    )
    .expect("write the helper module");
    let source = dir.join(format!("trusted.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "import clib\n\n\
         unsafe extern \"C\":\n\
         \x20   @ffi(param(text, borrowed, nul_terminated), link_name=\"strlen\")\n\
         \x20   safe fn length(text: cstr) -> usize\n\
         \x20   @ffi(immutable)\n    static frozen: i32\n\n\
         fn main():\n    println(clib.abs(-3), length(c\"abc\"))\n\
         \x20   unsafe:  # SAFETY(ffi): rand has no precondition\n        println(clib.rand() >= 0)\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let report = ember(&["tcb", &source_arg], &root);
    assert_eq!(report.exit, 0, "tcb failed:\n{}\n{}", report.stdout, report.stderr);
    let out = &report.stdout;
    let guaranteed = out.find("Guaranteed by the language").expect("the language's guarantees are stated");
    let supplied = out.find("Supplied from outside the language").expect("the supplied facts are a section of their own");
    assert!(guaranteed < supplied, "{out}");
    assert!(out.contains("C FFI (4 declarations):"), "{out}");
    let clib = out.find("  module clib").expect("grouped by module");
    let trusted = out.find("  module trusted\n    ").expect("grouped by module");
    assert!(out[clib..].contains("fn `abs` safe"), "{out}");
    assert!(out.contains("safe to call: a call within its contract cannot break memory safety (`safe fn`)  [asserted]  "), "{out}");
    assert!(out.contains("its effects: `effects=[]`  [asserted]  "), "{out}");
    assert!(out.contains(&format!("clib.{SOURCE_EXT}:2:10")), "{out}");
    assert!(out.contains("the same result on every run: `deterministic`  [asserted]"), "{out}");
    assert!(out.contains("fn `rand` unsafe to call"), "{out}");
    assert!(out[trusted..].contains("its signature matches the C declaration (`unsafe extern \"C\"`, C symbol \"strlen\")  [asserted]  "), "{out}");
    assert!(out.contains("its contract: `param(text, borrowed, nul_terminated)`  [asserted]  "), "{out}");
    assert!(out.contains("a `const` C object that never changes: `immutable`  [asserted]"), "{out}");
    assert!(!out.contains("[instrumented]") && !out.contains("[checked]") && !out.contains("[proven]"), "{out}");
    assert!(out.contains("block in `main`  SAFETY(ffi): rand has no precondition  [asserted]"), "{out}");
    let only = ember(&["tcb", "--module", "clib", &source_arg], &root);
    assert!(only.stdout.contains("C FFI (2 declarations):") && !only.stdout.contains("`length`"), "{}", only.stdout);
    let json = ember(&["tcb", "--json", &source_arg], &root);
    assert_eq!(json.exit, 0, "tcb --json failed:\n{}", json.stderr);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("tcb --json is JSON");
    let foreign = value["foreign"].as_array().expect("a foreign list");
    assert_eq!(foreign.len(), 4, "{}", json.stdout);
    assert!(foreign.iter().all(|decl| decl["facts"].as_array().is_some_and(|facts|
        facts.iter().all(|fact| fact["grade"] == "asserted" && fact["source"].is_string()))), "{}", json.stdout);
    assert!(value["guaranteed"].is_string(), "{}", json.stdout);
}

/// `[FFI-37]`, `[FFI-37a]`, `[FFI-37b]`, `[FFI-37c]`, `[FFI-37d]`, `[TCB-5]`, `[TCB-6]` —
/// `ember test --instrument-ffi` runs the tests with the allocation, lock, wait and I/O entry
/// points intercepted while a foreign function runs. A declared absence a run contradicts is
/// `E5053` and fails the run; one whose function no test called is `W5054` and counts for
/// nothing; the rest are graded `instrumented`, partial when a call site never ran, and the
/// report says what stays unobservable and which facts stay asserted. The evidence record names
/// its identity inputs (toolchain, test binary, each declaration's source and library) and its
/// environment; `ember tcb` shows the grades by it, says the record is stale once an identity
/// input changes, and only notes a record measured on another machine.
#[test]
fn instrumented_tests_grade_foreign_effect_facts() {
    let package = test_package("measured", "bin", "\
unsafe extern \"C\":
    @ffi(effects=[])
    safe fn abs(x: i32) -> i32
    @ffi(effects=[])
    safe fn imaxabs(x: i64) -> i64
    @ffi(param(text, borrowed, nul_terminated), effects=[])
    safe fn strlen(text: cstr) -> usize
    @ffi(effects=[])
    fn malloc(size: usize) -> *mut void
    @ffi(effects=[Alloc])
    fn free(block: *mut void)

@test
fn magnitudes():
    assert(abs(-5) == 5)
    assert(abs(7) == 7)

@test
fn lengths():
    length = strlen(c\"four\")
    if length > 10:
        assert(strlen(c\"never\") == 5)
    assert(length == 4)

@test
fn allocates():
    unsafe:  # SAFETY(ffi): the block is freed once, by the allocator that made it
        block = malloc(16)
        free(block)

fn main():
    println(imaxabs(-3))
");
    let out_dir = package.join("target");
    let out_arg = out_dir.to_string_lossy().into_owned();
    let run = ember(&["test", "--instrument-ffi", "--out-dir", &out_arg], &package);
    let text = format!("{}{}", run.stdout, run.stderr);
    assert_ne!(run.exit, 0, "a contradicted fact passed the run:\n{text}");
    assert!(run.stdout.contains("test result: ok. 3 passed"), "the tests themselves pass:\n{text}");
    for line in [
        "foreign effects, instrumented [FFI-37]:",
        "is structurally unobservable, and counts for nothing [FFI-37a]",
        "ownership, nullability, lifetime, aliasing and exception facts stay asserted [FFI-37d]",
        "`abs`: 2 calls from 2 of 2 call sites; no Alloc: instrumented; no Lock: instrumented; no Block: instrumented; no Io: instrumented",
        "`strlen`: 1 call from 1 of 2 call sites; no Alloc: instrumented, partial;",
        "`malloc`: 1 call from 1 of 1 call site; no Alloc: contradicted (E5053)",
        "`imaxabs`: 0 calls from 0 of 1 call site; no Alloc, no Lock, no Block, no Io: unexercised (W5054)",
    ] {
        assert!(run.stdout.contains(line), "missing `{line}`:\n{text}");
    }
    assert!(!run.stdout.contains("`free`: ") || run.stdout.contains("`free`: 1 call"), "{text}");
    assert!(run.stderr.contains("error[E5053]: an instrumented run contradicted `malloc`'s effects: it reached Alloc"), "{text}");
    assert!(run.stderr.contains("warning[W5054]: `imaxabs`'s effect facts were never exercised"), "{text}");
    assert!(!run.stderr.contains("`abs`"), "nothing contradicted `abs`:\n{text}");

    let record_path = package.join(format!(".{}/ffi-evidence/main.json", ember_branding::SYMBOL_PREFIX));
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&record_path).expect("the evidence record is written"))
        .expect("the evidence record is JSON");
    assert!(record["identity"]["toolchain"].is_string() && record["identity"]["test_binary"]["hash"].is_string(), "{record}");
    assert!(record["environment"]["machine"].is_string() && record["intercepted"].as_str().is_some_and(|text| text.contains("alloc:")), "{record}");
    let functions = record["functions"].as_array().expect("a function list");
    let abs = functions.iter().find(|function| function["name"] == "abs").expect("`abs` is recorded");
    assert!(abs["source_hash"].is_string() && abs["library"]["path"].as_str().is_some_and(|path| !path.is_empty()), "{abs}");
    assert_eq!(abs["facts"][0]["grade"], "instrumented", "{abs}");

    // `[TCB-5]` — `ember tcb` grades the facts by the record.
    let entry = package.join("src").join(ember_branding::source_file("main"));
    let entry_arg = entry.to_string_lossy().into_owned();
    let tcb = ember(&["tcb", &entry_arg], &package);
    assert_eq!(tcb.exit, 0, "{}{}", tcb.stdout, tcb.stderr);
    assert!(tcb.stdout.contains("no Alloc: instrumented, no Lock: instrumented, no Block: instrumented, no Io: instrumented [FFI-37c]"), "{}", tcb.stdout);
    assert!(tcb.stdout.contains("no Alloc: instrumented, partial (1 of 2 call sites ran)"), "{}", tcb.stdout);
    assert!(tcb.stdout.contains("instrumented: no counterexample was observed on the paths exercised, which is not that the fact holds"), "{}", tcb.stdout);
    assert!(tcb.stdout.contains("no Alloc: contradicted") && tcb.stdout.contains("no Alloc: unexercised"), "{}", tcb.stdout);
    assert!(!tcb.stdout.contains("not this machine"), "{}", tcb.stdout);

    // `[TCB-6]` — an environment input is reported, never a reason the record is stale.
    let elsewhere = std::fs::read_to_string(&record_path).expect("the record").replacen("\"machine\": \"", "\"machine\": \"elsewhere-", 1);
    std::fs::write(&record_path, elsewhere).expect("the record is writable");
    let moved = ember(&["tcb", &entry_arg], &package);
    assert!(moved.stdout.contains("not this machine: an environment input, reported, not a reason the record is stale [TCB-6]"), "{}", moved.stdout);
    assert!(moved.stdout.contains("no Io: instrumented [FFI-37c]") && !moved.stdout.contains("evidence stale"), "{}", moved.stdout);

    // `[TCB-6]` — an identity input changed (the declaring source): the facts stay asserted.
    let source = std::fs::read_to_string(&entry).expect("the source");
    std::fs::write(&entry, source.replace("strlen(c\"never\") == 5", "strlen(c\"nevermore\") == 9")).expect("the source is writable");
    let stale = ember(&["tcb", &entry_arg], &package);
    assert!(stale.stdout.contains("evidence stale, so its effect facts stay asserted: its declaration in "), "{}", stale.stdout);
    assert!(!stale.stdout.contains("[FFI-37c]"), "{}", stale.stdout);
}

/// `[CLI-13]` — `ember calls --foreign <item>` lists every foreign function the item can reach —
/// through calls, a method, another module, a function value it takes — with the chain to it and
/// the contract its declaration states; one it cannot reach is not listed, and a call through a
/// `dyn` value, whose callee nothing names, is said to be beyond the list.
#[test]
fn calls_foreign_lists_what_an_item_reaches_with_each_contract() {
    let root = workspace_root();
    let dir = temporary_directory("calls-foreign");
    std::fs::write(
        dir.join(format!("clib.{SOURCE_EXT}")),
        "unsafe extern \"C\":\n    @ffi(effects=[], deterministic)\n    pub safe fn abs(x: i32) -> i32\n    pub fn rand() -> i32\n",
    )
    .expect("write the helper module");
    let source = dir.join(format!("reach.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "import clib\n\n\
         unsafe extern \"C\":\n\
         \x20   @ffi(param(text, borrowed, nul_terminated), link_name=\"strlen\")\n\
         \x20   safe fn length(text: cstr) -> usize\n\
         \x20   @ffi(effects=[])\n    safe fn labs(x: i64) -> i64\n\
         \x20   safe fn imaxabs(x: i64) -> i64\n\n\
         interface Shape:\n    fn area(self) -> i64\n\n\
         struct Square:\n    side: i64\n\n\
         extend Square implements Shape:\n    fn area(self) -> i64:\n        return labs(self.side * self.side)\n\n\
         class Counter:\n    count: i32\n\n    fn bump(mut self) -> i32:\n        self.count += clib.abs(-1)\n        return self.count\n\n\
         fn noisy() -> i32:\n    unsafe:  # SAFETY(ffi): rand has no precondition\n        return clib.rand()\n\n\
         fn measure(shape: ref dyn Shape) -> i64:\n    return shape.area()\n\n\
         fn main():\n    counter = Counter(count = 0)\n    println(counter.bump(), length(c\"abc\"), noisy())\n\
         \x20   pick = imaxabs\n    println(pick(-4))\n    square = Square(side = 2)\n    println(measure(ref square))\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let report = ember(&["calls", "--foreign", "main", &source_arg], &root);
    assert_eq!(report.exit, 0, "calls failed:\n{}\n{}", report.stdout, report.stderr);
    let out = &report.stdout;
    assert!(out.contains("Foreign functions `main` can reach (4):"), "{out}");
    assert!(!out.contains("`labs`"), "`labs` is reached only through a `dyn` value: {out}");
    assert!(out.contains("  main -> length\n"), "{out}");
    assert!(out.contains("its contract: `param(text, borrowed, nul_terminated)`  [asserted]"), "{out}");
    assert!(out.contains("  main -> Counter.bump -> clib.abs\n"), "{out}");
    assert!(out.contains("its effects: `effects=[]`  [asserted]"), "{out}");
    assert!(out.contains("  main -> noisy -> clib.rand\n"), "{out}");
    assert!(out.contains("fn `rand` unsafe to call") && out.contains("its effects: none stated, so every effect"), "{out}");
    assert!(out.contains("  main -> imaxabs\n"), "a function value taken is reachable: {out}");
    assert!(out.contains("and what `measure` calls through a function or `dyn` value"), "{out}");
    let helper = ember(&["calls", "--foreign", "Counter.bump", &source_arg], &root);
    assert!(helper.stdout.contains("(1):") && helper.stdout.contains("Counter.bump -> clib.abs"), "{}", helper.stdout);
    let json = ember(&["calls", "--foreign", "noisy", "--json", &source_arg], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("calls --json is JSON");
    assert_eq!(value["foreign"][0]["symbol"], "rand", "{}", json.stdout);
    assert_eq!(value["foreign"][0]["declaration"]["safe"], false, "{}", json.stdout);
    let missing = ember(&["calls", "--foreign", "absent", &source_arg], &root);
    assert_ne!(missing.exit, 0);
}

/// `[CLI-12]` — `ember why --alloc <item>` prints the shortest call chain from the item to an
/// operation that allocates, whether or not the item carries a contract, and says so when it reaches
/// none; `--io` and the other effects likewise. With `--unsafe` it prints the obligation each unsafe
/// function on the chain states (`@safety`, `[UNS-7]`).
#[test]
fn why_prints_the_shortest_chain_to_an_effect() {
    let root = workspace_root();
    let dir = temporary_directory("why-effect");
    let source = dir.join(format!("why.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "fn grow(mut xs: Array[int]):\n    xs.push(1)\n\n\
         fn deep(mut xs: Array[int]):\n    grow(xs)\n\n\
         fn shallow(mut xs: Array[int]):\n    xs.push(2)\n\n\
         fn pure(x: int) -> int:\n    return x * 2\n\n\
         @safety(\"`n` is at least 1\")\n\
         unsafe fn first(p: *mut int, n: int) -> int:  # SAFETY: slot 0 exists, as `n` says\n    return read(p, 0)\n\n\
         fn through(p: *mut int) -> int:\n    unsafe:  # SAFETY: the caller's one slot\n        return first(p, 1)\n\n\
         fn peek() -> int:\n    value = 0\n    unsafe:  # SAFETY: one slot, written before it is read and freed once\n        p = alloc[int](1)\n        write(p, 0, 7)\n        value = first(p, 1)\n        free(p, 1)\n    return value\n\n\
         fn main():\n    xs: Array[int] = []\n    deep(xs)\n    shallow(xs)\n    println(pure(3), peek())\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let alloc = ember(&["why", "--alloc", "main", &source_arg], &root);
    assert_eq!(alloc.exit, 0, "why failed:\n{}\n{}", alloc.stdout, alloc.stderr);
    assert!(alloc.stdout.contains("`main` reaches `Alloc`:"), "{}", alloc.stdout);
    // The shortest chain: `shallow` pushes itself, `deep` only through `grow`.
    let deep = ember(&["why", "--alloc", "deep", &source_arg], &root);
    assert!(deep.stdout.contains("  deep -> grow -> `Array.push`"), "{}", deep.stdout);
    let pure = ember(&["why", "--alloc", "pure", &source_arg], &root);
    assert!(pure.stdout.contains("`pure` does not reach `Alloc`"), "{}", pure.stdout);
    let io = ember(&["why", "--io", "main", &source_arg], &root);
    assert!(io.stdout.contains("`main` reaches `Io`:") && io.stdout.contains("main -> console output"), "{}", io.stdout);
    let unsafe_chain = ember(&["why", "--unsafe", "through", &source_arg], &root);
    assert!(unsafe_chain.stdout.contains("  through -> first -> an `unsafe fn`"), "{}", unsafe_chain.stdout);
    assert!(unsafe_chain.stdout.contains("  `first` requires: `n` is at least 1"), "{}", unsafe_chain.stdout);
    let json = ember(&["why", "--alloc", "deep", "--json", &source_arg], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("why --json is JSON");
    assert_eq!(value["chains"][0]["chain"], serde_json::json!(["deep", "grow", "`Array.push`"]), "{}", json.stdout);
    let missing = ember(&["why", "--alloc", "absent", &source_arg], &root);
    assert_ne!(missing.exit, 0);
}

/// `[DET-9]` — `ember inspect --deterministic <item>` prints whether the item satisfies `[DET-1]`
/// and, if not, the shortest chain to each `Nondet` source; one that relies on a foreign function's
/// asserted determinism (`@ffi(deterministic)`) is reported as asserted, not verified.
#[test]
fn inspect_deterministic_names_each_nondeterministic_source() {
    let root = workspace_root();
    let dir = temporary_directory("inspect-deterministic");
    let source = dir.join(format!("det.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "import std.random\n\n\
         unsafe extern \"C\":\n    @ffi(effects=[], deterministic)\n    safe fn imaxabs(x: i64) -> i64\n\n\
         fn wave(x: f64) -> f64:\n    return x.sin()\n\n\
         fn noisy(x: f64) -> f64:\n    rng = random.Rng.from_entropy()\n    return wave(x) + rng.float()\n\n\
         fn total(xs: Span[int]) -> int:\n    t = 0\n    for x in xs:\n        t += x\n    return t\n\n\
         fn distance(a: i64, b: i64) -> i64:\n    return imaxabs(a - b)\n\n\
         fn main():\n    xs = [1, 2]\n    println(noisy(1.0), total(xs.as_span()), distance(3, 9))\n",
    )
    .expect("write the program");
    let source_arg = source.to_string_lossy().into_owned();
    let noisy = ember(&["inspect", "--deterministic", "noisy", &source_arg], &root);
    assert_eq!(noisy.exit, 0, "inspect failed:\n{}\n{}", noisy.stdout, noisy.stderr);
    assert!(noisy.stdout.contains("`noisy` does not satisfy [DET-1]: it reaches 2 `Nondet` source(s)"), "{}", noisy.stdout);
    assert!(noisy.stdout.contains("noisy -> wave -> a transcendental function from the platform library"), "{}", noisy.stdout);
    assert!(noisy.stdout.contains("the system random source"), "{}", noisy.stdout);
    let total = ember(&["inspect", "--deterministic", "total", &source_arg], &root);
    assert!(total.stdout.contains("`total` satisfies [DET-1]\n"), "{}", total.stdout);
    let distance = ember(&["inspect", "--deterministic", "distance", &source_arg], &root);
    assert!(distance.stdout.contains("`distance` satisfies [DET-1] on asserted foreign facts, not verified:"), "{}", distance.stdout);
    assert!(distance.stdout.contains("distance -> imaxabs (`@ffi(deterministic)` asserted)"), "{}", distance.stdout);
    let json = ember(&["inspect", "--deterministic", "total", "--json", &source_arg], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("--json is JSON");
    assert_eq!(value["deterministic"], true, "{}", json.stdout);
}

/// `[EFF-16]` — `ember inspect --safety` reports each function's `@nopanic(explicit)` beside the
/// `RuntimeCheck` kinds it reaches: a `@nopanic(explicit)` function may index (a bounds check is a
/// `RuntimeCheck`, allowed under it), and one free of checks too has no panic the model knows of.
#[test]
fn inspect_safety_reports_nopanic_explicit_beside_runtime_checks() {
    let root = workspace_root();
    let dir = temporary_directory("safety-nopanic");
    let source = dir.join(format!("nopanic.{SOURCE_EXT}"));
    std::fs::write(
        &source,
        "@nopanic(explicit)\nfn pick(xs: Span[int], i: int) -> int:\n    return xs[i]\n\n\
         @nopanic(explicit)\nfn twice(x: bool) -> bool:\n    return not x\n\n\
         fn main():\n    xs = [1, 2, 3]\n    println(pick(xs.as_span(), 1), twice(true))\n",
    )
    .expect("write the program");
    let out_dir = dir.join("out");
    let build = ember(&["build", &source.to_string_lossy(), "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(build.exit, 0, "build failed:\n{}", build.stderr);
    let table = out_dir.join("debug").join("inspect").join("nopanic.safety.json");
    let table = table.to_string_lossy().into_owned();
    let report = ember(&["inspect", "--safety", &table], &root);
    assert_eq!(report.exit, 0, "inspect failed:\n{}", report.stderr);
    let line = |name: &str| {
        report.stdout.lines().find(|line| line.trim_start().starts_with(&format!("{name} "))).unwrap_or_else(|| panic!("no line for `{name}`:\n{}", report.stdout)).to_string()
    };
    let pick = line("pick");
    assert!(pick.contains("@nopanic(explicit)") && pick.contains("Bounds") && !pick.contains("knows of"), "{pick}");
    let twice = line("twice");
    assert!(twice.contains("@nopanic(explicit)  RuntimeCheck: none  (no panic the effect model knows of)"), "{twice}");
    let main = line("main");
    assert!(!main.contains("@nopanic"), "{main}");
    let json = ember(&["inspect", "--safety", "--json", "--function", "twice", &table], &root);
    assert_eq!(json.exit, 0, "JSON inspect failed:\n{}", json.stderr);
    assert!(json.stdout.contains("\"functions\":[{\"function\":\"twice\",\"nopanic_explicit\":true,\"runtime_checks\":[]}]"), "{}", json.stdout);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[EXC-8]`–`[EXC-14]` — a copied class handle prevents static unique-handle
/// elision, but the receiver itself is stable across this counted loop. The
/// compiler therefore emits one checked loop-level interval for the `mut
/// self` calls' whole-object write (`[EXC-15]`) and reports its proof.
#[test]
fn stable_class_loop_access_is_reported_as_hoisted() {
    let root = workspace_root();
    let source = format!(
        "tests/conformance/EXC-8/accept_stable_class_receiver_loop.{SOURCE_EXT}"
    );
    for profile in ["debug", "development", "release"] {
        let out_dir = temporary_directory(&format!("hoisted-loop-safety-{profile}"));
        let run = ember(
            &[
                "build",
                &source,
                "--profile",
                profile,
                "--out-dir",
                &out_dir.to_string_lossy(),
            ],
            &root,
        );
        assert_eq!(run.exit, 0, "{profile} build failed:\n{}", run.stderr);

        let side_table = out_dir
            .join(profile)
            .join("inspect")
            .join("accept_stable_class_receiver_loop.safety.json");
        let json = std::fs::read_to_string(&side_table)
            .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
        assert!(
            json.contains("\"classification\":\"DYNAMIC_HOISTED_LOOP\""),
            "{profile} stable loop was not classified as hoisted:\n{json}"
        );
        assert!(
            json.contains("\"check_site\":\"preheader\""),
            "{profile} hoisted loop lacks its preheader check record:\n{json}"
        );
        assert!(
            json.contains("\"protected_interval\":\"loop\""),
            "{profile} hoisted loop lacks its protected interval record:\n{json}"
        );
        assert!(
            json.contains("\"proof\":\"stable_receiver_direct_call\""),
            "{profile} hoisted loop lacks its proof record:\n{json}"
        );

        let c_path = out_dir
            .join(profile)
            .join("c")
            .join("accept_stable_class_receiver_loop.c");
        let c = std::fs::read_to_string(&c_path)
            .unwrap_or_else(|error| panic!("{}: {error}", c_path.display()));
        let main_symbol = ember_branding::mangled("main");
        let bump_symbol = ember_branding::mangled("Counter_bump");
        let main_start = c
            .find(&format!("void {main_symbol}(void)"))
            .expect("main is emitted");
        let bump_start = c
            .rfind(&format!("void {bump_symbol}("))
            .expect("method is emitted");
        let main_c = &c[main_start..bump_start];
        let begin_write = ember_branding::runtime("object_begin_write");
        let end_write = ember_branding::runtime("object_end_write");
        assert_eq!(
            main_c.matches(&begin_write).count(),
            1,
            "{profile} main must have one preheader write check:\n{main_c}"
        );
        assert_eq!(
            main_c.matches(&end_write).count(),
            1,
            "{profile} main must have one postheader write release:\n{main_c}"
        );

        let side_table_arg = side_table.to_string_lossy().into_owned();
        let report = ember(&["inspect", "--safety", &side_table_arg], &root);
        assert_eq!(report.exit, 0, "{profile} inspect failed:\n{}", report.stderr);
        assert!(
            report.stdout.contains("DYNAMIC_HOISTED_LOOP")
                && report.stdout.contains("proof: stable_receiver_direct_call"),
            "{profile} inspect did not identify the hoisted proof:\n{}",
            report.stdout
        );
    }
}

/// `[EXC-10]`, `[EXC-12]` — each nest's loop around the inner loop holds the
/// inner loop's access, and the report names that loop and its proof.
#[test]
fn an_outer_loop_reports_the_inner_access_it_holds() {
    let root = workspace_root();
    let name = "accept_an_outer_loop_holds_its_inner_loops_access";
    let source = format!("tests/conformance/EXC-10/{name}.{SOURCE_EXT}");
    for profile in ["debug", "development", "release"] {
        let out_dir = temporary_directory(&format!("outer-loop-safety-{profile}"));
        let run = ember(&["build", &source, "--profile", profile, "--out-dir", &out_dir.to_string_lossy()], &root);
        assert_eq!(run.exit, 0, "{profile} build failed:\n{}", run.stderr);
        let side_table = out_dir.join(profile).join("inspect").join(format!("{name}.safety.json"));
        let report = ember(&["inspect", "--safety", &side_table.to_string_lossy()], &root);
        assert_eq!(report.exit, 0, "{profile} inspect failed:\n{}", report.stderr);
        for line in [26, 30, 34] {
            assert!(
                report.stdout.contains(&format!("{name}.{SOURCE_EXT}:{line}:17  proof: inner_loop_reused")),
                "{profile} the loop on line {line} does not hold its inner access:\n{}",
                report.stdout
            );
        }
    }
}

/// ADR-079 — for MSVC, a loop nest that writes separate lists it never reads
/// runs in a function of its own whose list parameters are `restrict`, the
/// counts the program fixes written in; for clang the nest stays in place.
#[test]
fn loop_nests_over_separate_lists_get_restrict_functions_for_msvc() {
    let root = workspace_root();
    let source = format!("tests/conformance/OPT-2/accept_a_loop_nest_over_separate_lists_keeps_its_results.{SOURCE_EXT}");
    let main_loop = format!("{}_loop0(", ember_branding::mangled("main"));
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:
{}", msvc.stderr);
    assert!(msvc.stdout.contains(&format!("EMBER_NOINLINE void {main_loop}")), "no loop function for MSVC:
{}", msvc.stdout);
    assert!(msvc.stdout.contains("* restrict _1_ptr"), "the loop function's lists are not restrict:
{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE void").count(), 4, "two nests move (prototype and definition each)");
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:
{}", clang.stderr);
    assert!(!clang.stdout.contains(&main_loop), "clang keeps the nest in place:
{}", clang.stdout);
}

/// D-571 — a lambda passed as a callable value that is set once is called
/// directly, inlined, and its overflow checks proved away by the loop's
/// facts, on every C compiler: `a08_closures` calls nothing through the
/// value and checks nothing in its loop.
#[test]
fn a_known_callable_value_is_called_directly() {
    let root = workspace_root();
    let source = format!("bench/programs/a08_closures.{SOURCE_EXT}");
    for cc in ["msvc", "clang", "gcc"] {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc}: C failed:\n{}", c.stderr);
        let main = c_definition(&c.stdout, &ember_branding::mangled("main"));
        assert!(!main.contains(".native))("), "{cc}: the loop calls through the value:\n{main}");
        assert!(!main.contains("ck_mul_i64("), "{cc}: the lambda's check stayed:\n{main}");
    }
}

/// D-569 — on Windows a function only ever called directly takes each `str`
/// or span parameter as its pointer and length (the platform passes a
/// 16-byte struct through memory); one used as a value keeps the view whole.
/// Elsewhere the C keeps every view whole.
#[test]
fn view_parameters_go_as_two_words_on_windows() {
    let root = workspace_root();
    let source = format!("tests/run-pass/views_passed_as_two_words.{SOURCE_EXT}");
    for cc in if cfg!(windows) { &["msvc", "clang"][..] } else { &["clang"][..] } {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc}: C failed:\n{}", c.stderr);
        let count = ember_branding::mangled("count_spaces");
        let first = ember_branding::mangled("first_len");
        let split = format!("{count}(const unsigned char* _1_p, size_t _1_n, int64_t _2)");
        assert_eq!(c.stdout.contains(&split), cfg!(windows), "{cc}: count_spaces's text parameter:\n{}", c.stdout);
        assert!(!c.stdout.contains(&format!("{first}(const unsigned char*")), "{cc}: a function used as a value keeps its view whole");
    }
}

/// D-557, D-568 — `[DSJ-3]`: the loop over views `assert_disjoint` proved
/// apart runs in a `restrict` function for every C compiler, as do
/// `split_at`'s halves. `[SIMD-3]`: the `Err` arm's views and two view
/// parameters prove nothing, so nothing of `unproved` moves, and only the `Ok`
/// arm of `scale` does (its loop and the copy loop versioning makes of it).
#[test]
fn views_proved_apart_get_restrict_functions_and_nothing_else_does() {
    let root = workspace_root();
    let source = format!("tests/conformance/DSJ-3/accept_views_proved_apart_loop_in_a_restrict_function.{SOURCE_EXT}");
    for cc in ["msvc", "clang", "gcc"] {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc} C failed:\n{}", c.stderr);
        // A kernel's name as code uses it, followed by its call or parameter list: the backtrace's
        // table of names (`[RT-4]`) writes it as a string too, which is no use.
        let kernels = |name: &str| {
            let stem = format!("{}_loop", ember_branding::mangled(name));
            c.stdout
                .match_indices(&stem)
                .filter(|(at, _)| {
                    let rest = &c.stdout[at + stem.len()..];
                    rest.trim_start_matches(|ch: char| ch.is_ascii_digit()).starts_with('(')
                })
                .count()
        };
        // Each kernel is named twice: its prototype and its definition, and
        // once more where it is called.
        assert_eq!(kernels("scale"), 6, "{cc}: the Ok arm's loop and its versioned copy, and nothing of the Err arm:\n{}", c.stdout);
        assert!(kernels("halves") > 0, "{cc}: split_at's halves are apart:\n{}", c.stdout);
        assert_eq!(kernels("unproved"), 0, "{cc}: two view parameters prove nothing:\n{}", c.stdout);
        assert!(!c.stdout.contains("EMBER_NOINLINE EMBER_NOINLINE"), "{cc}: a kernel keeps its function's own hint");
    }
}

/// `[CT-4]`, `[CT-2]` — a compile-time value is kept with its inputs: the
/// next build of the same source reuses it, and a changed input (here an
/// environment variable `comptime.env` read) computes it again.
#[test]
fn a_compile_time_value_is_cached_with_its_inputs() {
    let directory = temporary_directory("comptime-cache");
    let source = directory.join(format!("main.{SOURCE_EXT}"));
    std::fs::write(&source, "fn main():
    println(comptime(comptime.env(\"EMBER_CT_CACHE_PROBE\")))
").expect("the source is written");
    let out = directory.join("target");
    let (source, out) = (source.to_string_lossy().to_string(), out.to_string_lossy().to_string());
    let args = ["run", source.as_str(), "--out-dir", out.as_str()];
    let first = ember_with_env(&args, &directory, "EMBER_CT_CACHE_PROBE", "one");
    assert_eq!(first.exit, 0, "the first build failed:
{}", first.stderr);
    assert!(first.stdout.contains("Some('one')"), "{}", first.stdout);
    let kept: Vec<PathBuf> = std::fs::read_dir(directory.join("target").join("debug").join("comptime"))
        .expect("the evaluator's directory")
        .flatten()
        .flat_map(|program| std::fs::read_dir(program.path().join("cache")).into_iter().flatten().flatten())
        .map(|entry| entry.path())
        .collect();
    assert!(!kept.is_empty(), "no value was kept");
    let entry = std::fs::read_to_string(&kept[0]).expect("a kept value");
    assert!(entry.contains("env ") && entry.contains("EMBER_CT_CACHE_PROBE"), "the input is not recorded with the value:
{entry}");
    let again = ember_with_env(&args, &directory, "EMBER_CT_CACHE_PROBE", "one");
    assert!(again.stdout.contains("Some('one')"), "{}", again.stdout);
    let changed = ember_with_env(&args, &directory, "EMBER_CT_CACHE_PROBE", "two");
    assert!(changed.stdout.contains("Some('two')"), "a changed input was not read again: {}", changed.stdout);
}

/// `[RT-5]` — the runtime's names derive from one constant, the branding
/// prefix: the generated header defines it and spells every name literally
/// (`tools/test_runtime_generation.py` regenerates it under another prefix).
#[test]
fn the_runtime_header_spells_its_names_from_the_one_prefix() {
    let root = workspace_root();
    let prefix = ember_branding::SYMBOL_PREFIX;
    let runtime = root.join("runtime").join(format!("{prefix}_rt"));
    let header = std::fs::read_to_string(runtime.join("include").join(format!("{prefix}_rt.h"))).expect("the runtime header");
    let source = std::fs::read_to_string(runtime.join("src").join(format!("{prefix}_rt.c"))).expect("the runtime source");
    assert!(header.contains(&format!("#define {}_SYMBOL_PREFIX {prefix}", prefix.to_uppercase())), "the header defines the prefix");
    assert!(header.contains(&format!("int {prefix}_rt_init_v2(")), "the header names the entry point literally");
    assert!(source.contains(&format!("#include \"{prefix}_rt.h\"")), "the source includes the header by its prefixed name");
}

/// `[SIMD-3]` (c) — distinct `SoA` columns are apart: the loop over two
/// columns of one `SoA` runs in a function of its own whose column pointers
/// are `restrict`, for every C compiler.
#[test]
fn distinct_soa_columns_get_restrict_functions() {
    let root = workspace_root();
    let source = format!("tests/conformance/SIMD-3/accept_distinct_soa_columns_loop_in_a_restrict_function.{SOURCE_EXT}");
    for cc in ["msvc", "clang", "gcc"] {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc} C failed:
{}", c.stderr);
        let kernel = format!("{}_loop", ember_branding::mangled("step"));
        assert!(c.stdout.contains(&kernel), "{cc}: the columns' loop did not move:
{}", c.stdout);
        assert!(c.stdout.contains("float* restrict _1_ptr, float* restrict _2_ptr"), "{cc}: the columns are not restrict:
{}", c.stdout);
    }
}

/// ADR-099 — for clang and gcc, a list an innermost loop only pushes onto is
/// kept in a local for the loop and written back on every way out
/// (`list_locals.rs`): the C writes the object's list back from a local. Not
/// for MSVC, which keeps the local in memory (8 times slower). (The
/// conformance test checks the results, including loops where something else
/// reads the list.)
#[test]
fn a_list_a_loop_only_pushes_onto_is_kept_in_a_local() {
    let root = workspace_root();
    let source = format!("tests/conformance/OPT-2/accept_a_list_a_loop_only_pushes_onto_keeps_every_push.{SOURCE_EXT}");
    // A write of `items` from a local outside `Log`'s constructor (its definition, the last mention),
    // whose own write (`[CLS-2]`: through a temporary, after the field's drop flag) is no write-back.
    let written_back = |c: &str| {
        let init = c.rfind("Log_init(struct").unwrap_or(c.len());
        let (before, after) = c.split_at(init);
        let after = after.find("\n}\n").map_or("", |end| &after[end..]);
        before.contains("->items = _") || after.contains("->items = _")
    };
    let gcc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "gcc"], &root);
    assert_eq!(gcc.exit, 0, "C failed:\n{}", gcc.stderr);
    assert!(written_back(&gcc.stdout), "no list is written back from a local:\n{}", gcc.stdout);
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "C failed:\n{}", msvc.stderr);
    assert!(!written_back(&msvc.stdout), "a list is kept in a local for MSVC:\n{}", msvc.stdout);
}

/// ADR-099 — behind the tests of the classes a program knows, an interface
/// call's table search is the runtime's out-of-line one: the inline search
/// is a loop, and a loop inside the caller's kept gcc from unrolling it.
#[test]
fn a_devirtualised_interface_call_searches_out_of_line() {
    let root = workspace_root();
    let source = format!("tests/run-pass/class_interface_handle.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development"], &root);
    assert_eq!(c.exit, 0, "C failed:\n{}", c.stderr);
    let out_of_line = format!("{}(", ember_branding::runtime("itable_lookup_slow"));
    let inline = format!("{}(", ember_branding::runtime("itable_lookup"));
    assert!(c.stdout.contains(&out_of_line), "the fallback is not out of line:\n{}", c.stdout);
    assert!(!c.stdout.contains(&inline), "an inline table search remains:\n{}", c.stdout);
}

/// D-579 — `[LEX-1]`, `[LEX-2]`: a source file that is not UTF-8 is `E0001`, whether it
/// is the one compiled or a module it imports; a byte-order mark is accepted
/// and ignored. A conformance file cannot hold such bytes, so they are
/// written here.
#[test]
fn a_source_file_that_is_not_utf8_is_e0001() {
    let root = workspace_root();
    let directory = temporary_directory("not-utf8");
    let bad = directory.join(ember_branding::source_file("bad"));
    std::fs::write(&bad, b"fn main():\n    println(\"\xff\")\n").unwrap();
    let checked = ember(&["check", &bad.to_string_lossy()], &root);
    assert_ne!(checked.exit, 0);
    assert!(checked.stderr.contains("error[E0001]: "), "{}", checked.stderr);
    let helper = directory.join(ember_branding::source_file("helper"));
    std::fs::write(&helper, b"pub fn one() -> int:\n    return 1  # \xfe\n").unwrap();
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&entry, b"import helper\n\nfn main():\n    println(helper.one())\n").unwrap();
    let imported = ember(&["check", &entry.to_string_lossy()], &root);
    assert_ne!(imported.exit, 0);
    assert!(imported.stderr.contains("error[E0001]: "), "{}", imported.stderr);
    let marked = directory.join(ember_branding::source_file("marked"));
    std::fs::write(&marked, b"\xef\xbb\xbffn main():\n    println(1)\n").unwrap();
    let out = directory.join("out");
    let run = ember(&["run", &marked.to_string_lossy(), "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "1\n");
    // `[LEX-2]` — CRLF and LF are one line ending: a multiline string holds
    // LF, and indentation reads the same.
    let crlf = directory.join(ember_branding::source_file("crlf"));
    std::fs::write(&crlf, b"fn main():\r\n    text = \"\"\"\r\n    a\r\n    b\r\n    \"\"\"\r\n    println(text.len())\r\n").unwrap();
    let run = ember(&["run", &crlf.to_string_lossy(), "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "4\n");
}

/// D-450 — `[STD-10]`, `[TXT-1]`, `[TXT-2]`: `input()` returns a `String`,
/// which is valid UTF-8, and only a `_lossy` form may substitute U+FFFD, so a
/// line that is not UTF-8 is a console failure and panics; a valid line with
/// characters of several bytes reads whole. A conformance file cannot hold
/// bytes that are not UTF-8, so they are piped in here.
#[test]
fn input_panics_on_a_line_that_is_not_utf8() {
    let root = workspace_root();
    let source = format!("tests/conformance/STD-10/accept_input_reads_a_line.{SOURCE_EXT}");
    let bad = ember_with_input(&["run", &source], &root, b"\xe2\x82\n42\nend\n");
    // A panic exits non-zero (3 on Windows, 1 on Linux), as the run-fail
    // runner checks.
    assert_ne!(bad.exit, 0, "no panic:\n{}{}", bad.stdout, bad.stderr);
    assert!(bad.stderr.contains("input: the line is not valid UTF-8"), "another panic:\n{}", bad.stderr);
    let good = ember_with_input(&["run", &source], &root, "caf\u{e9}\n42\nend\n".as_bytes());
    assert_eq!(good.exit, 0, "{}", good.stderr);
    assert_eq!(good.stdout, "name? hi caf\u{e9}\nage? 2\nend\n");
}

/// D-446 — `[CG-C-1]`: a signed `+`, `-` or `*` (negation is `0 - x`) reaching C
/// unchecked is done in its width's unsigned type, so wrapping is never C's
/// undefined signed overflow (gcc -O2 miscompiled a wrapping `i * k` in a
/// loop); MSVC and clang happened to keep it, so the C itself is the test.
#[test]
fn wrapping_arithmetic_is_unsigned_in_c() {
    let root = workspace_root();
    let source = format!("tests/conformance/TYP-8/accept_wrapping_arithmetic_in_a_loop_is_never_undefined.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "gcc"], &root);
    assert_eq!(c.exit, 0, "C failed:\n{}", c.stderr);
    for wrapped in [
        "(int64_t)((uint64_t)(_1) * (uint64_t)(6364136223846793005LL))",
        "(int32_t)((uint32_t)(",
        "(int16_t)((uint32_t)(",
        "(int64_t)((uint64_t)(0LL) - (uint64_t)(",
    ] {
        assert!(c.stdout.contains(wrapped), "no `{wrapped}`: wrapping arithmetic is signed in C:\n{}", c.stdout);
    }
}

/// D-445 — a printing helper takes its value as `T const*`: for a class
/// handle that is `struct X* const*`, which `&handle` converts to, where
/// `const struct X**` is an incompatible pointer, an error from gcc 14 on (an
/// older gcc warns, which no run sees).
#[test]
fn printing_a_class_handle_passes_a_pointer_c_accepts() {
    let root = workspace_root();
    let source = format!("tests/conformance/TYP-36/accept_a_class_handle_prints_its_class_and_address.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--cc", "gcc"], &root);
    assert_eq!(c.exit, 0, "C failed:
{}", c.stderr);
    assert!(c.stdout.contains("* const* v)"), "the helper does not take `T const*`:
{}", c.stdout);
    assert!(!c.stdout.contains("** v)"), "a helper takes `const T**`:
{}", c.stdout);
}

/// ADR-098 — for gcc, a loop over two lists or more that writes one runs in a
/// function of its own whose list parameters are `restrict`, nest or not, and
/// whether or not it reads back what it writes (gcc does not turn loops
/// around, so MSVC's conditions on that do not apply): every loop of the
/// separate-lists program moves, and three of the four nests reading back
/// their list, all but the one whose addition keeps its overflow check.
#[test]
fn loops_over_separate_lists_get_restrict_functions_for_gcc() {
    let root = workspace_root();
    let separate = format!("tests/conformance/OPT-2/accept_a_loop_nest_over_separate_lists_keeps_its_results.{SOURCE_EXT}");
    let gcc = ember(&["build", &separate, "--emit", "c", "--profile", "development", "--cc", "gcc"], &root);
    assert_eq!(gcc.exit, 0, "gcc C failed:
{}", gcc.stderr);
    assert_eq!(gcc.stdout.matches("EMBER_NOINLINE void").count(), 6, "three loops move (prototype and definition each):
{}", gcc.stdout);
    assert!(gcc.stdout.contains("* restrict _1_ptr"), "the loop function's lists are not restrict:
{}", gcc.stdout);
    let reading_back = format!("tests/conformance/OPT-2/accept_a_loop_nest_adding_the_same_each_round_keeps_its_results.{SOURCE_EXT}");
    let gcc = ember(&["build", &reading_back, "--emit", "c", "--profile", "development", "--cc", "gcc"], &root);
    assert_eq!(gcc.exit, 0, "gcc C failed:
{}", gcc.stderr);
    assert_eq!(gcc.stdout.matches("EMBER_NOINLINE void").count(), 6, "three nests move for gcc:
{}", gcc.stdout);
    let msvc = ember(&["build", &reading_back, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE void").count(), 2, "one nest moves for MSVC");
}

/// ADR-103 — for MSVC, a small counted loop with a check or a call is written
/// eight turns to a pass (gcc's `-funroll-loops` limits), with each call that
/// takes an object's address taking it directly (`bump(&t)`: MSVC does not
/// inline a call through a pointer variable set just before it, in such a
/// loop); a loop carrying a running total keeps one turn to a pass, and
/// clang's C keeps the loops as they were.
#[test]
fn msvc_writes_small_checked_loops_several_turns_a_pass() {
    let root = workspace_root();
    let source = format!("tests/conformance/CTL-3/accept_a_counted_loop_runs_every_turn_in_any_pass.{SOURCE_EXT}");
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:\n{}", msvc.stderr);
    assert!(msvc.stdout.contains(" > 7u; ) {"), "no loop runs eight turns to a pass:\n{}", msvc.stdout);
    assert!(msvc.stdout.contains("_bump(&_"), "the call does not take the object's address directly:\n{}", msvc.stdout);
    let total_of = format!("{}(", ember_branding::mangled("total_of"));
    let start = msvc
        .stdout
        .match_indices(&total_of)
        .map(|(at, _)| at)
        .find(|&at| msvc.stdout[at..].split('\n').next().is_some_and(|line| line.ends_with('{')))
        .expect("total_of is defined");
    let end = start + msvc.stdout[start..].find("\n}\n").expect("total_of ends");
    assert!(!msvc.stdout[start..end].contains("u; ) {"), "the running total's loop was unrolled:\n{}", &msvc.stdout[start..end]);
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:\n{}", clang.stderr);
    assert!(!clang.stdout.contains("u; ) {"), "clang's C was unrolled:\n{}", clang.stdout);
}

/// ADR-080 — a loop nest reading back the list it writes moves for MSVC only
/// where every round adds the same whole number to each element: of the
/// four nests, only the one adding `a[i]` each round (the sum kept below
/// 1024) moves; mixing in the round, keeping the sum below 1000, and an
/// addition with its overflow check stay in place.
#[test]
fn loop_nests_adding_the_same_each_round_get_restrict_functions_for_msvc() {
    let root = workspace_root();
    let source = format!("tests/conformance/OPT-2/accept_a_loop_nest_adding_the_same_each_round_keeps_its_results.{SOURCE_EXT}");
    let main_loop = format!("{}_loop0(", ember_branding::mangled("main"));
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:
{}", msvc.stderr);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE void").count(), 2, "one nest moves (prototype and definition):
{}", msvc.stdout);
    let start = msvc.stdout.rfind(&format!("EMBER_NOINLINE void {main_loop}")).expect("the loop function is defined");
    let kernel = &msvc.stdout[start..start + msvc.stdout[start..].find("
}
").expect("the loop function ends")];
    assert!(kernel.contains("& 1023LL") && !kernel.contains(" ^ ") && !kernel.contains("& 1000LL"), "the wrong nest moved:
{kernel}");
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:
{}", clang.stderr);
    assert!(!clang.stdout.contains(&main_loop), "clang keeps the nest in place:
{}", clang.stdout);
}

/// ADR-084 — for MSVC, a loop nest carrying one running value over views runs
/// in a function of its own that returns it: the nest over two views, the
/// nest whose value comes in from before it, and the decimal one. Two running
/// values, a nest that writes a list, and a single loop stay; clang keeps
/// every loop.
#[test]
fn loop_nests_carrying_a_running_value_get_their_own_function_for_msvc() {
    let root = workspace_root();
    let source = format!("tests/conformance/OPT-2/accept_a_loop_carrying_a_running_value_keeps_its_results.{SOURCE_EXT}");
    let main_loop = format!("{}_loop", ember_branding::mangled("main"));
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:\n{}", msvc.stderr);
    assert_eq!(msvc.stdout.matches(&format!("EMBER_NOINLINE int64_t {main_loop}")).count(), 4, "two whole-number nests move (prototype and definition each):\n{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches(&format!("EMBER_NOINLINE double {main_loop}")).count(), 2, "the decimal nest moves:\n{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE ").count(), 6, "nothing else moves:\n{}", msvc.stdout);
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:\n{}", clang.stderr);
    assert!(!clang.stdout.contains(&main_loop), "clang keeps every loop in place:\n{}", clang.stdout);
}

/// ADR-086 — for MSVC, which does not unroll a loop setting a variable of the
/// whole function, a counted loop runs on a block-local copy of each number it
/// sets (`double _7 = _7_in;` inside, `_7 = _7_in;` after): five totals, two
/// lines each. clang lost a range it had used with the copies, so its C keeps
/// the totals where they are.
#[test]
fn loop_totals_are_block_local_copies_for_msvc() {
    let root = workspace_root();
    let source = format!("tests/conformance/CTL-3b/accept_a_running_total_keeps_its_results.{SOURCE_EXT}");
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:
{}", msvc.stderr);
    assert_eq!(msvc.stdout.lines().filter(|line| line.ends_with("_in;")).count(), 10, "five totals copied in and back:
{}", msvc.stdout);
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:
{}", clang.stderr);
    assert!(!clang.stdout.lines().any(|line| line.ends_with("_in;")), "clang keeps the totals in place:
{}", clang.stdout);
}

/// `[PHIL-5]` — at the end of `main` the process ends: development and release
/// builds leave the list of objects with no `drop` to the operating system
/// (one `free` fewer in `main`), and keep every drop that runs a `drop`
/// method; debug builds free everything, for their leak check.
#[test]
fn exit_drops_that_only_free_memory_go_in_release() {
    let root = workspace_root();
    let source = format!("tests/conformance/DRP-2/accept_drops_at_the_end_of_main.{SOURCE_EXT}");
    let main_symbol = ember_branding::mangled("main");
    let vec_free = format!("{}(", ember_branding::runtime("vec_free"));
    let mut frees = Vec::new();
    for profile in ["debug", "development", "release"] {
        let out_dir = temporary_directory(&format!("exit-drops-{profile}"));
        let run = ember(&["build", &source, "--profile", profile, "--out-dir", &out_dir.to_string_lossy()], &root);
        assert_eq!(run.exit, 0, "{profile} build failed:
{}", run.stderr);
        let c_path = out_dir.join(profile).join("c").join("accept_drops_at_the_end_of_main.c");
        let c = std::fs::read_to_string(&c_path).unwrap_or_else(|error| panic!("{}: {error}", c_path.display()));
        let start = c.find(&format!("void {main_symbol}(void) {{")).expect("main is emitted");
        let main_c = &c[start..start + c[start..].find("
}
").expect("main ends")];
        // The interface list's elements are released: `Loud` has a `drop`.
        assert!(main_c.contains("_di0"), "{profile} main lost a drop that runs a `drop` method:
{main_c}");
        frees.push(main_c.matches(&vec_free).count());
        let _ = std::fs::remove_dir_all(&out_dir);
    }
    assert_eq!(frees[1] + 1, frees[0], "development frees one list fewer than debug: {frees:?}");
    assert_eq!(frees[2], frees[1], "release drops what development drops: {frees:?}");
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
    let out_dir = temporary_directory("static-elision-side-table");
    let run = ember(
        &[
            "build",
            &format!("tests/conformance/HEAP-5/accept_unique_shared_get_mut_elides_access.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir.to_string_lossy(),
        ],
        &root,
    );
    assert_eq!(run.exit, 0, "build failed:\n{}", run.stderr);

    let side_table = out_dir
        .join("debug")
        .join("inspect")
        .join("accept_unique_shared_get_mut_elides_access.safety.json");
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

/// `[EXC-3]`, `[EXC-3a]` — a check that nothing held anywhere in the program
/// could make fail is removed, and recorded in the safety side table.
#[test]
fn a_check_nothing_held_could_fail_is_removed_and_recorded() {
    let root = workspace_root();
    let source = format!("tests/conformance/EXC-3/accept_a_check_nothing_held_could_fail_is_removed.{SOURCE_EXT}");
    let out_dir = temporary_directory("no-conflicting-hold");
    let run = ember(&["build", &source, "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(run.exit, 0, "build failed:
{}", run.stderr);
    let side_table = out_dir
        .join("debug")
        .join("inspect")
        .join("accept_a_check_nothing_held_could_fail_is_removed.safety.json");
    let json = std::fs::read_to_string(&side_table)
        .unwrap_or_else(|error| panic!("{}: {error}", side_table.display()));
    assert!(json.contains("\"reason\":\"no_conflicting_hold\""), "missing removed checks: {json}");
    let emitted = ember(&["build", &source, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "C emission failed: {}", emitted.stderr);
    assert!(!emitted.stdout.contains("field_check_"), "a check nothing could fail survived");
}

/// `[EXC-19]` — an access nothing can overlap is one check, not a counted
/// begin and end. Here a write held elsewhere keeps the read's check.
#[test]
fn an_access_nothing_can_overlap_is_only_checked() {
    let root = workspace_root();
    let source = format!("tests/conformance/EXC-3/run_fail_a_read_during_a_write_held_elsewhere.{SOURCE_EXT}");
    let emitted = ember(&["build", &source, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains("field_check_read("), "the read is not a check");
    assert!(!emitted.stdout.contains("field_begin_read("), "the read is still counted");
}

/// `[EXC-15]`, `[EXC-19]` — a `mut self` call to a quiet method is one check
/// of the object's words; nothing marks them for the call.
#[test]
fn a_quiet_mut_self_call_is_only_checked() {
    let root = workspace_root();
    let source = format!("tests/conformance/EXC-19/run_fail_a_quiet_mut_self_call_while_a_field_is_viewed.{SOURCE_EXT}");
    let emitted = ember(&["build", &source, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains("object_check_failed("), "the call is not a check");
    assert!(!emitted.stdout.contains("object_begin_write("), "the call still marks the object");
}

/// `[EXC-19]`, `[OBJ-1]` — each class level's access words stand side by side
/// before its fields, a zero word after an odd count.
#[test]
fn access_words_are_packed_before_their_fields() {
    let root = workspace_root();
    let source = format!("tests/conformance/EXC-19/accept_packed_access_words_in_a_base_and_a_derived_class.{SOURCE_EXT}");
    let emitted = ember(&["build", &source, "--emit", "c"], &root);
    assert_eq!(emitted.exit, 0, "C emission failed: {}", emitted.stderr);
    assert!(emitted.stdout.contains("    uint32_t _access_a;
    uint32_t _access_pad0;
"), "base words not packed");
    assert!(emitted.stdout.contains("    uint32_t _access_b;
    uint32_t _access_c;
"), "derived words not packed");
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
        &["explain", "--cycle", &package_arg, "scene.Node.next"],
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
            .contains("scene.Node.next -> scene.Node"),
        "qualified explanation did not use Ember qualification:\n{}",
        qualified.stdout
    );

    let missing_class = ember(
        &["explain", "--cycle", &package_arg, "scene.Missing"],
        &root,
    );
    assert_ne!(missing_class.exit, 0, "missing class unexpectedly resolved");
    assert!(
        missing_class.stderr.contains("cannot find class `scene.Missing`"),
        "missing class used the wrong diagnostic:\n{}",
        missing_class.stderr
    );

    let missing_field = ember(
        &["explain", "--cycle", &package_arg, "scene.Node.missing"],
        &root,
    );
    assert_ne!(missing_field.exit, 0, "missing field unexpectedly resolved");
    assert!(
        missing_field.stderr.contains("has no field `missing`"),
        "missing field used the wrong diagnostic:\n{}",
        missing_field.stderr
    );

    // `[GRM-24]` (0.9.9) — `.` is the one path separator, here as in source.
    let colons = ember(
        &["explain", "--cycle", &package_arg, "scene::Node.next"],
        &root,
    );
    assert_ne!(colons.exit, 0, "a `::` target unexpectedly resolved");
    assert!(
        colons.stderr.contains("use '.' for paths"),
        "a `::` target did not say to use '.':\n{}",
        colons.stderr
    );

    let ambiguous = ember(&["explain", "--cycle", &package_arg, "Node"], &root);
    assert_ne!(ambiguous.exit, 0, "ambiguous class unexpectedly resolved");
    assert!(
        ambiguous.stderr.contains("scene.Node") && ambiguous.stderr.contains("ecs.Node"),
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
        !invalid_root.stderr.contains("scene.Node"),
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

    let mixed = ember(
        &[
            "check",
            "--json",
            &format!(
                "tests/conformance/WK-5/warning_mixed_cycle_suggests_first_weakable_edge.{SOURCE_EXT}"
            ),
        ],
        &root,
    );
    assert_eq!(mixed.exit, 0, "mixed-cycle check failed:\n{}", mixed.stderr);
    assert!(
        mixed.stdout.contains("\"replacement\":\"Weak[Owner]\""),
        "a later direct class edge in the cycle should receive the safe Weak replacement:\n{}",
        mixed.stdout
    );
    assert!(
        mixed.stdout.contains("replace `Child.parent` with `Weak[Owner]`"),
        "the suggestion should name the first safely weakenable edge:\n{}",
        mixed.stdout
    );
    assert!(
        !mixed.stdout.contains("replace `Owner.child`"),
        "a Shared ownership contract must not receive an automatic Weak rewrite:\n{}",
        mixed.stdout
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
    let out_dir = temporary_directory("leak-check");
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
    // `[WK-4]` names the edge to weaken; `[WK-10]` offers no rewrite of its
    // `Option` type.
    assert!(
        report.stderr.contains("suggested weak edge: Node.next (no automatic rewrite")
            && !report.stderr.contains("replace with"),
        "leak-check did not name the edge, or offered a Weak rewrite for an Option ownership contract:\n{}",
        report.stderr
    );
    assert!(
        report.stderr.contains("weak Node.previous"),
        "leak-check omitted the weak ownership vocabulary:\n{}",
        report.stderr
    );
}

/// `[BLD-9]` — `--timings` reports each stage's time on standard error,
/// module by module where a stage runs so (reading and parsing each module),
/// and `--timings=json` the same as one JSON line; without the flag nothing.
#[test]
fn timings_report_each_stage_and_module() {
    let root = workspace_root();
    let source = format!("tests/conformance/RC-6/accept_count_operations_left_in_loops_are_listed.{SOURCE_EXT}");
    let out_dir = temporary_directory("timings");
    let out = out_dir.to_string_lossy().into_owned();
    let built = ember(&["build", &source, "--out-dir", &out, "--timings=json"], &root);
    assert_eq!(built.exit, 0, "build failed:\n{}", built.stderr);
    let line = built.stderr.lines().find(|line| line.starts_with("{\"timings\"")).expect("a JSON timings line");
    let value: serde_json::Value = serde_json::from_str(line).expect("the timings are JSON");
    let stages: Vec<(String, String)> = value["timings"]
        .as_array()
        .expect("a list of stages")
        .iter()
        .map(|entry| (entry["stage"].as_str().unwrap().to_string(), entry["module"].as_str().unwrap().to_string()))
        .collect();
    for (stage, module) in [
        ("parse", "main"),
        ("read and parse", "std.core"),
        ("check", "all modules"),
        ("lower", "all modules"),
        ("C generation", "all modules"),
        ("C compile and link", "all modules"),
    ] {
        assert!(
            stages.iter().any(|(s, m)| s == stage && m == module),
            "no {stage} of {module}: {stages:?}"
        );
    }
    assert!(value["total_ms"].as_f64().is_some_and(|ms| ms > 0.0), "{line}");
    let checked = ember(&["check", &source, "--timings"], &root);
    assert_eq!(checked.exit, 0, "{}", checked.stderr);
    assert!(checked.stderr.contains("timings (ms):") && checked.stderr.contains("  total"), "{}", checked.stderr);
    assert!(!checked.stderr.contains("C compile and link"), "check compiled C:\n{}", checked.stderr);
    let quiet = ember(&["check", &source], &root);
    assert!(!quiet.stderr.contains("timings"), "{}", quiet.stderr);
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// `[DSP-5]` — `ember build --emit-optimization-report` lists, on standard
/// error, each virtual call the whole program made direct, at its location; a
/// call on a final class (`[DSP-1]`) and one two bodies answer are not listed,
/// and a program with none says so.
#[test]
fn the_optimization_report_lists_each_direct_call() {
    let root = workspace_root();
    let source = format!("tests/conformance/DSP-5/accept_a_call_with_one_implementation_is_direct.{SOURCE_EXT}");
    let built = ember(&["build", &source, "--emit", "c", "--emit-optimization-report"], &root);
    assert_eq!(built.exit, 0, "build failed:\n{}", built.stderr);
    let listed: Vec<&str> = built.stderr.lines().filter(|line| line.contains("([DSP-5])")).collect();
    assert_eq!(listed.len(), 2, "{}", built.stderr);
    let text = std::fs::read_to_string(root.join(&source)).expect("the program reads");
    let at = |needle: &str| text.lines().position(|line| line.contains(needle)).expect("the call is in the program") + 1;
    for (call, says) in [
        ("s.sides()", "calls `Shape.sides` directly, the only implementation for class `Shape`"),
        ("a.legs()", "calls `Dog.legs` directly, the only implementation for class `Animal`"),
    ] {
        let line = format!(".{SOURCE_EXT}:{}:", at(call));
        assert!(
            listed.iter().any(|listed| listed.contains(&line) && listed.contains(says)),
            "missing {says:?} at {line}:\n{}",
            built.stderr
        );
    }
    assert!(!built.stdout.contains("optimization report"), "the report went into the C:\n{}", built.stdout);
    let final_only = format!("tests/conformance/DSP-1/accept_a_call_on_a_final_class_is_direct.{SOURCE_EXT}");
    let none = ember(&["build", &final_only, "--emit", "c", "--emit-optimization-report"], &root);
    assert_eq!(none.exit, 0, "{}", none.stderr);
    assert!(none.stderr.contains("optimization report:\n  none"), "{}", none.stderr);
}

/// `[SIMD-2]` — `--emit-optimization-report` says of each `@simd` loop
/// whether it is in vectorisable form and, when it is not, why (the first
/// clause of `[SIMD-5]` it fails). `[SIMD-6]` — a build that compiles the C
/// then prints the host C compiler's own vectorisation report, after Ember's
/// verdicts and labelled as its opinion; `--emit c` compiles nothing, so it
/// has none, and neither is ever an error.
#[test]
fn the_optimization_report_gives_each_simd_loop_its_verdict_then_the_host_compilers() {
    let root = workspace_root();
    let source = format!("tests/conformance/SIMD-2/accept_simd_is_a_request_that_never_changes_meaning.{SOURCE_EXT}");
    let text = std::fs::read_to_string(root.join(&source)).expect("the program reads");
    let at = |needle: &str| text.lines().position(|line| line.contains(needle)).expect("the loop is in the program") + 1;
    let out_dir = temporary_directory("simd_report");
    let out = out_dir.to_string_lossy().into_owned();
    let built = ember(&["build", &source, "--profile", "development", "--out-dir", &out, "--emit-optimization-report"], &root);
    assert_eq!(built.exit, 0, "build failed:\n{}", built.stderr);
    for (header, says) in [
        ("for i in 0..xs.len():", "is in vectorisable form"),
        ("for i in 0..idx.len():", "not in vectorisable form: it reaches memory other than a local or a view at the loop counter plus a constant"),
        ("for x in xs:", "not in vectorisable form: it keeps a floating-point running total `s`"),
        ("for c in t.chars():", "not in vectorisable form: its trip count is not known before entry"),
        ("if xs[i] >= limit:", "not in vectorisable form: it has more than one exit"),
    ] {
        // The verdict is at the `for` (the line before an `if` that leaves).
        let line = if header.starts_with("if") { at(header) - 1 } else { at(header) };
        let place = format!(".{SOURCE_EXT}:{line}:");
        assert!(
            built.stderr.lines().any(|listed| listed.contains(&place) && listed.contains("`@simd` loop") && listed.contains(says)),
            "missing {says:?} at {place}:\n{}",
            built.stderr
        );
    }
    let verdicts = built.stderr.find("`@simd` loop").expect("the verdicts are reported");
    let host = built.stderr.find("host C compiler's vectorisation report").expect("the host compiler's report follows");
    assert!(verdicts < host, "the host compiler's opinion came first:\n{}", built.stderr);
    assert!(built.stderr[host..].contains("its opinion only"), "{}", built.stderr);
    let c = ember(&["build", &source, "--emit", "c", "--emit-optimization-report"], &root);
    assert_eq!(c.exit, 0, "{}", c.stderr);
    assert!(c.stderr.contains("`@simd` loop") && !c.stderr.contains("host C compiler"), "{}", c.stderr);
    let _ = std::fs::remove_dir_all(&out_dir);
}

/// `[STD-1]` — every function in `std.core`, `std.mem`, `std.math`, `std.simd` and `std.arena` is
/// `@noalloc` unless its documentation says it allocates: a copy of `std` whose `core` gains a
/// function that allocates with nothing saying so stops the build with `E4001`; the same function
/// whose documentation says it allocates, a generic one, and the same function in `std.thread`
/// (not one of the five) do not.
#[test]
fn std_functions_are_noalloc_unless_documented() {
    let root = workspace_root();
    let copy = temporary_directory("std_noalloc");
    let std_src = root.join("std").join("src");
    let copied = copy.join("src");
    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).expect("the copy's directory");
        for entry in std::fs::read_dir(from).expect("std's sources") {
            let entry = entry.expect("an entry");
            let target = to.join(entry.file_name());
            if entry.path().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), &target).expect("a std file copies");
            }
        }
    }
    let program = copy.join(format!("main.{SOURCE_EXT}"));
    std::fs::write(&program, "import std.thread\n\nfn main():\n    println(1)\n").expect("the program writes");
    let program = program.to_string_lossy().into_owned();
    let with = |module: &str, extra: &str| {
        let _ = std::fs::remove_dir_all(&copied);
        copy_tree(&std_src, &copied);
        let file = copied.join(format!("{module}.{SOURCE_EXT}"));
        let text = std::fs::read_to_string(&file).expect("the module reads");
        std::fs::write(&file, format!("{text}\n{extra}")).expect("the module writes");
        ember_with_env(&["check", &program], &root, &ember_branding::std_path_var(), &copied.to_string_lossy())
    };
    let leaky = "pub fn leaky_text() -> String:\n    return String.from(\"made\")\n";
    let undocumented = with("core", leaky);
    assert_ne!(undocumented.exit, 0, "an undocumented allocation in std.core was accepted:\n{}", undocumented.stderr);
    assert!(
        undocumented.stderr.contains("E4001") && undocumented.stderr.contains("leaky_text"),
        "{}",
        undocumented.stderr
    );
    let documented = with("core", &format!("## It allocates the text it gives back.\n{leaky}"));
    assert_eq!(documented.exit, 0, "a documented allocation was refused:\n{}", documented.stderr);
    let generic = with("core", "pub fn leaky_list[T](owned x: T) -> Array[T]:\n    return [x]\n");
    assert_eq!(generic.exit, 0, "a generic function was held to it:\n{}", generic.stderr);
    let elsewhere = with("thread", leaky);
    assert_eq!(elsewhere.exit, 0, "a module outside the five was held to it:\n{}", elsewhere.stderr);
}

/// `[RC-6]` — `ember inspect --counts` lists the retains and releases a build
/// left inside loops, each with its reason; a loop that reads borrowed
/// handles has none.
#[test]
fn inspect_counts_lists_count_operations_left_in_loops() {
    let root = workspace_root();
    let out_dir = temporary_directory("inspect-counts");
    let out = out_dir.to_string_lossy().into_owned();
    let stem = "accept_count_operations_left_in_loops_are_listed";
    let program = format!("tests/conformance/RC-6/{stem}.{SOURCE_EXT}");
    let built = ember(&["build", &program, "--out-dir", &out], &root);
    assert_eq!(built.exit, 0, "build failed:\n{}", built.stderr);
    let table = out_dir.join("debug").join("inspect").join(format!("{stem}.counts.json"));
    let table = table.to_string_lossy().into_owned();
    let report = ember(&["inspect", "--counts", &table], &root);
    assert_eq!(report.exit, 0, "inspect failed:\n{}", report.stderr);
    for needle in [
        "retain 2",
        "release 1",
        "`node` is pushed into a list, which keeps a count of its own",
        "`other` is stored into a field or element of `holder`, which keeps a count of its own",
        "the value in a field or element of `holder` is released as a store replaces it ([OWN-5])",
        "in the loop at",
    ] {
        assert!(report.stdout.contains(needle), "missing {needle:?}:\n{}", report.stdout);
    }
    let json = ember(&["inspect", "--counts", "--json", &table], &root);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("--json prints JSON");
    assert_eq!(value["operations"].as_array().map(Vec::len), Some(3), "{}", json.stdout);
    let other = ember(&["inspect", "--counts", "--function", "nothing_here", &table], &root);
    assert!(other.stdout.contains("none"), "a function filter kept another's operations:\n{}", other.stdout);
    // A loop reading through borrowed handles makes no count operation.
    let quiet = format!("tests/conformance/RC-2b/accept_a_handle_read_within_one_expression_is_not_retained.{SOURCE_EXT}");
    assert_eq!(ember(&["build", &quiet, "--out-dir", &out], &root).exit, 0);
    let quiet_table = out_dir.join("debug").join("inspect").join("accept_a_handle_read_within_one_expression_is_not_retained.counts.json");
    let quiet = ember(&["inspect", "--counts", &quiet_table.to_string_lossy()], &root);
    assert!(quiet.stdout.contains("none"), "a borrowed loop reported counts:\n{}", quiet.stdout);
    let both = ember(&["inspect", "--counts", "--cycle", &table], &root);
    assert!(both.exit != 0 && both.stderr.contains("only one report kind"), "{}", both.stderr);
    let elided = ember(&["inspect", "--counts", "--elided-only", &table], &root);
    assert!(elided.exit != 0 && elided.stderr.contains("applies only to `--safety`"), "{}", elided.stderr);
}

/// `[SEL-1]` (ODR-108) — `ember inspect --alloc` also reports the storage mechanism each
/// declaration uses: the row of IX.0's table its declared type picks, or `value`, `view`,
/// `callable` or `raw pointer` for the types the table has no row for. A lambda's environment is
/// the compiler's and is not listed.
#[test]
fn inspect_alloc_reports_the_mechanism_each_declaration_uses() {
    let root = workspace_root();
    let program = format!("tests/conformance/PHIL-2/accept_every_allocating_form_is_listed.{SOURCE_EXT}");
    let json = ember(&["inspect", "--alloc", "--json", &program], &root);
    assert_eq!(json.exit, 0, "inspect failed:\n{}", json.stderr);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("--json prints JSON");
    let declarations = value["declarations"].as_array().expect("a declarations array");
    let uses = |kind: &str, owner: &str, name: &str, mechanism: &str| {
        declarations.iter().any(|d| d["kind"] == kind && d["owner"] == owner && d["name"] == name && d["mechanism"] == mechanism)
    };
    for (kind, owner, name, mechanism) in [
        ("type", "", "Node", "class"),
        ("field", "Node", "value", "value"),
        ("parameter", "total", "xs", "view"),
        ("local", "total", "sum", "value"),
        ("local", "main", "node", "class"),
        ("local", "main", "xs", "Array, Map, Set, String"),
        ("local", "main", "ages", "Array, Map, Set, String"),
        ("local", "main", "boxed", "Box[T]"),
        ("local", "main", "shared", "Shared[T]"),
        ("local", "main", "many", "callable"),
    ] {
        assert!(uses(kind, owner, name, mechanism), "[SEL-1] {kind} {owner}.{name} should use {mechanism:?}:\n{}", json.stdout);
    }
    assert!(!declarations.iter().any(|d| d["owner"].as_str().is_some_and(|o| o.starts_with("lambda in"))), "[SEL-1] a lambda's environment is listed:\n{}", json.stdout);
    let text = ember(&["inspect", "--alloc", "--function", "total", &program], &root);
    assert_eq!(text.exit, 0, "{}", text.stderr);
    assert!(text.stdout.contains("Storage mechanisms:") && text.stdout.contains("total parameter `xs`: Span[i64] -> view"), "{}", text.stdout);
    assert!(!text.stdout.contains("main local"), "a function filter kept another's declarations:\n{}", text.stdout);
}

/// `[PHIL-2]`, `[CLO-10]` — `ember inspect --alloc` lists every place a
/// program allocates, by function and line, with what allocates there: each
/// allocating form `[PHIL-2]` names, and an owned callable value whose
/// captured state is larger than three words (`[CLO-10]`); a function that
/// allocates nothing, and three words of state, are not listed.
#[test]
fn inspect_alloc_lists_every_allocating_form() {
    let root = workspace_root();
    let program = format!("tests/conformance/PHIL-2/accept_every_allocating_form_is_listed.{SOURCE_EXT}");
    let json = ember(&["inspect", "--alloc", "--json", &program], &root);
    assert_eq!(json.exit, 0, "inspect failed:
{}", json.stderr);
    let value: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("--json prints JSON");
    let sites = value["sites"].as_array().expect("a sites array");
    let listed = |function: &str, line: u32, what: &str| {
        sites.iter().any(|site| {
            site["function"] == function
                && site["source"].as_str().is_some_and(|source| source.contains(&format!(":{line}:")))
                && site["allocates"].as_array().is_some_and(|all| all.iter().any(|op| op.as_str().is_some_and(|op| op.contains(what))))
        })
    };
    for (function, line, what) in [
        ("four", 16, "captured state is larger than three words"),
        ("main", 28, "a new class instance"),
        ("main", 29, "a new `Array`"),
        ("main", 30, "a new `String`"),
        ("main", 31, "a new `Array`"),
        ("main", 32, "`Array.push`"),
        ("main", 33, "a new `String`"),
        ("main", 34, "an f-string"),
        ("main", 35, "a new `Array`"),
        ("main", 36, "std.collections.Map.index_set`"),
        ("main", 37, "std.collections.Set.add`"),
        ("main", 38, "`Box(…)`"),
        ("main", 39, "`Shared(…)`"),
        ("main", 40, "`Array.push`"),
        ("main", 41, "std.collections.Map.insert`"),
        ("main", 42, "std.collections.Set.add`"),
    ] {
        assert!(listed(function, line, what), "[PHIL-2] {function} line {line} should list {what:?}:
{}", json.stdout);
    }
    for quiet in ["three", "total"] {
        assert!(!sites.iter().any(|site| site["function"] == quiet), "[CLO-10] `{quiet}` allocates nothing:
{}", json.stdout);
    }
    let text = ember(&["inspect", "--alloc", "--function", "four", &program], &root);
    assert_eq!(text.exit, 0, "{}", text.stderr);
    assert!(text.stdout.contains("Function: four") && text.stdout.contains(":16:12: a callable value whose captured state is larger than three words"), "{}", text.stdout);
    assert!(!text.stdout.contains("main "), "a function filter kept another's sites:
{}", text.stdout);
    let small = format!("tests/conformance/CLO-10/accept_small_captured_state_in_noalloc_code.{SOURCE_EXT}");
    let none = ember(&["inspect", "--alloc", &small], &root);
    assert!(none.exit == 0 && none.stdout.contains("  none"), "[CLO-10] three words allocate nothing:
{}{}", none.stdout, none.stderr);
    let elided = ember(&["inspect", "--alloc", "--elided-only", &program], &root);
    assert!(elided.exit != 0 && elided.stderr.contains("applies only to `--safety`"), "{}", elided.stderr);
    let both = ember(&["inspect", "--alloc", "--cycle", &program], &root);
    assert!(both.exit != 0 && both.stderr.contains("only one report kind"), "{}", both.stderr);
}

/// `[WK-15]` — the debug leak report is on for `ember run` by default; the
/// flag turns it off, and the other profiles have it only when asked.
#[test]
fn debug_runs_report_leaks_unless_turned_off() {
    let root = workspace_root();
    let out_dir = temporary_directory("leak-default");
    let out_dir = out_dir.to_string_lossy().into_owned();
    let program = format!("tests/conformance/WK-15/accept_a_debug_run_reports_a_leaked_cycle_by_default.{SOURCE_EXT}");
    let run = |extra: &[&str]| {
        let mut arguments = vec!["run", program.as_str(), "--out-dir", out_dir.as_str()];
        arguments.extend_from_slice(extra);
        ember(&arguments, &root)
    };
    let default = run(&[]);
    assert!(default.exit == 0 && default.stderr.contains("warning[L3017]"), "no default report:\n{}", default.stderr);
    let off = run(&["--no-leak-check"]);
    assert!(off.exit == 0 && !off.stderr.contains("runtime ownership cycle"), "--no-leak-check reported:\n{}", off.stderr);
    for profile in ["development", "release"] {
        let quiet = run(&["--profile", profile]);
        assert!(quiet.exit == 0 && !quiet.stderr.contains("runtime ownership cycle"), "{profile} reported unasked:\n{}", quiet.stderr);
        let asked = run(&["--profile", profile, "--leak-check"]);
        assert!(asked.exit == 0 && asked.stderr.contains("runtime ownership cycle"), "{profile} --leak-check silent:\n{}", asked.stderr);
    }
    let both = run(&["--leak-check", "--no-leak-check"]);
    assert!(both.exit != 0 && both.stderr.contains("contradict"), "both flags accepted:\n{}", both.stderr);
    let build = ember(&["build", program.as_str(), "--no-leak-check", "--out-dir", out_dir.as_str()], &root);
    assert!(build.exit != 0 && build.stderr.contains("only valid with `ember run`"), "build took --no-leak-check:\n{}", build.stderr);
}

fn assert_reported_node_components(report: &str, expected_count: usize) {
    let components = report.split("runtime ownership cycle:\n").skip(1).collect::<Vec<_>>();
    assert_eq!(
        components.len(),
        expected_count,
        "expected {expected_count} separate runtime SCC reports:\n{report}"
    );
    for component in components {
        let objects = component
            .lines()
            .find(|line| line.starts_with("  objects:"))
            .expect("component report includes its objects");
        assert_eq!(objects.matches("Node@").count(), 3, "expected three objects in SCC:\n{component}");
        assert!(component.contains("strong edges: 3"), "expected three internal strong edges:\n{component}");
        assert_eq!(
            component.lines().filter(|line| line.starts_with("    strong Node.next:")).count(),
            3,
            "expected each Node.next edge in the SCC:\n{component}"
        );
        assert!(component.contains("statically predicted: yes"), "expected static correlation:\n{component}");
    }
}

#[test]
fn leak_check_reports_a_three_object_strong_component() {
    let root = workspace_root();
    let out_dir = temporary_directory("three-node-cycle");
    let out_dir = out_dir.to_string_lossy().into_owned();
    let report = ember(
        &[
            "run",
            "--leak-check",
            &format!("tests/conformance/WK-8/accept_runtime_three_object_cycle.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir,
        ],
        &root,
    );
    assert_eq!(report.exit, 0, "leak-check run failed:\n{}", report.stderr);
    assert_reported_node_components(&report.stderr, 1);
}

#[test]
fn leak_check_reports_two_independent_strong_components() {
    let root = workspace_root();
    let out_dir = temporary_directory("two-independent-node-cycles");
    let out_dir = out_dir.to_string_lossy().into_owned();
    let report = ember(
        &[
            "run",
            "--leak-check",
            &format!("tests/conformance/WK-8/accept_runtime_two_independent_cycles.{SOURCE_EXT}"),
            "--out-dir",
            &out_dir,
        ],
        &root,
    );
    assert_eq!(report.exit, 0, "leak-check run failed:\n{}", report.stderr);
    assert_reported_node_components(&report.stderr, 2);
}

#[test]
fn leak_check_reports_a_shared_payload_cycle_without_static_overclaim() {
    let root = workspace_root();
    let out_dir = temporary_directory("shared-leak-check");
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

/// The C compiler this run uses, as a test's `#$ c-compilers:` line names it: `msvc`, `clang`
/// (`clang-cl` too) or `gcc`.
fn c_compiler_name() -> &'static str {
    let requested = std::env::var(ember_branding::cc_var()).ok();
    match Toolchain::detect(requested.as_deref()) {
        Ok(Toolchain::Msvc { .. }) => "msvc",
        Ok(Toolchain::Gcc(_)) => "gcc",
        _ => "clang",
    }
}

fn check_file(path: &Path, root: &Path) {
    let source = std::fs::read_to_string(path).expect("the test file is readable");
    let expectations = parse_expectations(&source);
    if !expectations.c_compilers.is_empty() && !expectations.c_compilers.iter().any(|name| name == c_compiler_name()) {
        return;
    }
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
    // Keep this case's output through all profiles and assertions, then remove
    // it even when an assertion unwinds. Short unique names also fit MSVC's
    // path limit and keep concurrent cases with the same filename apart.
    let out_dir = temporary_directory(&relative);
    let profiles = profiles(&expectations);

    // A `compile-fail` test must be rejected, with the diagnostics it names.
    if !expectations.errors.is_empty() || expectations.kind.as_deref() == Some("compile-fail") {
        for profile in &profiles {
            let (exit, rendered) = assert_exact_diagnostics(&relative, profile, root, &expectations);
            assert_ne!(
                exit, 0,
                "{relative} [{profile}]: expected compilation to fail, but it succeeded"
            );
            assert_diagnostic_helps(&relative, &rendered, &expectations);
        }
        return;
    }

    // `[TST-1]` — a program that compiles produces exactly the warnings and
    // lints it names, and none it does not.
    if expectations.warnings.is_empty() {
        let (_, rendered) = assert_exact_diagnostics(&relative, profiles[0], root, &expectations);
        if !expectations.helps.is_empty() || !expectations.forbidden_helps.is_empty() {
            assert_diagnostic_helps(&relative, &rendered, &expectations);
        }
    } else {
        for profile in &profiles {
            let (_, rendered) = assert_exact_diagnostics(&relative, profile, root, &expectations);
            assert_diagnostic_helps(&relative, &rendered, &expectations);
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
        // The driver adds the profile's own folder (`[BLD-5]`).
        let out_dir_arg = out_dir.to_string_lossy().into_owned();
        let mut arguments = vec!["run", relative.as_str(), "--out-dir", &out_dir_arg, "--profile", profile];
        if !expectations.args.is_empty() {
            arguments.push("--");
            arguments.extend(expectations.args.iter().map(String::as_str));
        }
        // `[TST-27]` — the C gate: an accepted program's C compiles with warnings as errors, on each
        // C compiler CI runs the suite with; a C compiler error on one is a compiler defect.
        let run = ember_gated(&arguments, root, expectations.stdin.as_ref().map(|input| input.as_bytes()));
        for needle in &expectations.stderr {
            assert!(
                run.stderr.contains(needle.as_str()),
                "{relative} [{profile}]: expected standard error to contain {needle:?}\nstderr:\n{}",
                run.stderr
            );
        }

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
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the test directory is readable")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e == ember_branding::SOURCE_EXT)
        })
        .collect();
    entries.sort();
    report_failures(name, &run_cases(&entries, |path| check_file(path, &root)));
    entries.len()
}

/// Cases running at once across every test function in this binary. Cargo
/// runs the functions side by side, and each would otherwise start a worker
/// per core of its own.
static RUNNING: Mutex<usize> = Mutex::new(0);
static FINISHED: Condvar = Condvar::new();

fn cores() -> usize {
    let available = std::thread::available_parallelism().map_or(4, |n| n.get());
    // Honour the standard test runner's explicit worker limit inside a case
    // sweep too; --test-threads alone only serializes the outer test functions.
    std::env::var("RUST_TEST_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|&limit| limit > 0)
        .map_or(available, |limit| limit.min(available))
}

/// A place among the `cores()` cases that may run at once, held until drop.
struct Permit;

impl Permit {
    fn take() -> Permit {
        let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
        while *running >= cores() {
            running = FINISHED.wait(running).unwrap_or_else(PoisonError::into_inner);
        }
        *running += 1;
        Permit
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) -= 1;
        FINISHED.notify_one();
    }
}

/// Run `case` over every file, one per core, and return the failures in the
/// files' order. A failing case is recorded rather than stopping the run, so
/// one run reports every failing case rather than the first.
fn run_cases(files: &[PathBuf], case: impl Fn(&Path) + Sync) -> Vec<String> {
    let next = AtomicUsize::new(0);
    let mut failures: Vec<(usize, String)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..cores().min(files.len()))
            .map(|_| {
                scope.spawn(|| {
                    let mut failed = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(path) = files.get(index) else { return failed };
                        let _permit = Permit::take();
                        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| case(path)));
                        if let Err(payload) = outcome {
                            let message = payload
                                .downcast_ref::<String>()
                                .cloned()
                                .or_else(|| payload.downcast_ref::<&str>().map(|text| text.to_string()))
                                .unwrap_or_else(|| "a case panicked with a non-text payload".to_string());
                            failed.push((index, message));
                        }
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().expect("a worker catches its cases' panics"))
            .collect()
    });
    failures.sort_by_key(|(index, _)| *index);
    failures.into_iter().map(|(_, message)| message).collect()
}

fn report_failures(what: &str, failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{} failing case(s) in {what}:\n\n{}",
        failures.len(),
        failures.join("\n\n----------\n\n")
    );
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
            let directory = temporary_directory("fmt");
            let scratch = directory.join(ember_branding::source_file("once"));
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

/// `[FMT-2]` — the formatter writes a lambda whose body is one expression in
/// the `=>` form: a block that only returns a value, and the `:` form inside
/// brackets. A body of more than that, or one with a comment, keeps its form;
/// the result formats to itself and parses to the same tree (`[FMT-1]`).
#[test]
fn formatter_writes_one_expression_lambdas_in_the_arrow_form() {
    let root = workspace_root();
    let directory = temporary_directory("fmt-arrow-lambdas");
    let scratch = directory.join(ember_branding::source_file("main"));
    std::fs::write(
        &scratch,
        "fn apply(f: fn(int) -> int, x: int) -> int:\n    return f(x)\n\n\nfn main():\n    add = fn(x: int) -> int:\n        return x + 1\n    keep = fn(x: int) -> int:\n        # a note\n        return x\n    two = fn(x: int) -> int:\n        y = x * 2\n        return y\n    println(apply(fn(x): x * 3, 2), apply(fn(x): return x - 1, 2), add(1), keep(2), two(3))\n    println(apply(fn(x) => x, 5))\n",
    )
    .expect("the fixture is writable");
    let path = scratch.to_string_lossy().into_owned();
    let before = ember(&["build", &path, "--emit", "ast"], &root);
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    assert_eq!(
        once.stdout,
        "fn apply(f: fn(int) -> int, x: int) -> int:\n    return f(x)\n\n\nfn main():\n    add = fn(x: int) -> int => x + 1\n    keep = fn(x: int) -> int:\n        # a note\n        return x\n    two = fn(x: int) -> int:\n        y = x * 2\n        return y\n    println(apply(fn(x) => x * 3, 2), apply(fn(x) => x - 1, 2), add(1), keep(2), two(3))\n    println(apply(fn(x) => x, 5))\n",
        "[FMT-2]"
    );
    std::fs::write(&scratch, &once.stdout).expect("the formatted fixture is writable");
    let twice = ember(&["fmt", &path], &root);
    assert_eq!(once.stdout, twice.stdout, "[FMT-1] fmt(fmt(x)) differs from fmt(x)");
    let after = ember(&["build", &path, "--emit", "ast"], &root);
    assert_eq!(before.stdout, after.stdout, "[FMT-1] parse(fmt(x)) differs from parse(x)");
    let checked = ember(&["check", &path], &root);
    assert_eq!(checked.exit, 0, "[FMT-2] the formatted program does not check:\n{}", checked.stderr);
}

#[test]
fn formatter_keeps_a_raw_reserved_name_raw() {
    // `[LEX-14]`, `[FMT-1]` — the tree keeps a name, not its spelling: a reserved word written raw
    // stays raw (it was a keyword bare), a raw name that is no keyword is written bare, and after `.`
    // `is` and `not` are method names bare (`[LEX-15]`).
    let root = workspace_root();
    let directory = temporary_directory("fmt-raw-names");
    let scratch = directory.join(ember_branding::source_file("main"));
    std::fs::write(&scratch, "struct Rec:\n    r#type: int\n    r#is: bool\n    r#plain: int\n    fn is(self) -> bool:\n        return self.r#is\n\nfn r#is() -> int:\n    return 1\n\nfn r#match(r#type: int) -> int:\n    r#let = r#type + 1\n    return r#let\n\nfn main():\n    rec = Rec(r#type=2, r#is=true, r#plain=3)\n    o: Option[Rec] = Some(Rec(r#type=5, r#is=false, r#plain=6))\n    println(rec.r#type, rec.is(), o?.is(), o?.r#type, r#is(), r#match(4))\n").expect("the fixture is writable");
    let path = scratch.to_string_lossy().into_owned();
    let before = ember(&["build", &path, "--emit", "ast"], &root);
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    assert_eq!(once.stdout, "struct Rec:\n    r#type: int\n    r#is: bool\n    plain: int\n\n    fn is(self) -> bool:\n        return self.is\n\n\nfn r#is() -> int:\n    return 1\n\n\nfn r#match(r#type: int) -> int:\n    r#let = r#type + 1\n    return r#let\n\n\nfn main():\n    rec = Rec(r#type=2, r#is=true, plain=3)\n    o: Option[Rec] = Some(Rec(r#type=5, r#is=false, plain=6))\n    println(rec.r#type, rec.is(), o?.is(), o?.r#type, r#is(), r#match(4))\n", "[LEX-14]");
    std::fs::write(&scratch, &once.stdout).expect("the formatted fixture is writable");
    let twice = ember(&["fmt", &path], &root);
    assert_eq!(once.stdout, twice.stdout, "[FMT-1] fmt(fmt(x)) differs from fmt(x)");
    let after = ember(&["build", &path, "--emit", "ast"], &root);
    assert_eq!(before.stdout, after.stdout, "[FMT-1] parse(fmt(x)) differs from parse(x)");
    let checked = ember(&["check", &path], &root);
    assert_eq!(checked.exit, 0, "[LEX-14] the formatted program does not check:\n{}", checked.stderr);
}

/// `[DIA-22]` — a diagnostic a callee's signature causes is at the call, with the callee's
/// declaration as a secondary span: the function for the count of arguments and an unknown name,
/// the parameter for an argument of the wrong type.
#[test]
fn a_call_diagnostic_points_at_the_callee_declaration() {
    let root = workspace_root();
    let program = format!("tests/conformance/DIA-22/reject_calls_against_their_signatures.{SOURCE_EXT}");
    let out = ember(&["check", &program], &root);
    assert_ne!(out.exit, 0, "the program should not check");
    for label in [
        "`area` is declared here",
        "parameter `h` is declared `i64` here",
        "`scale` is declared here",
        "parameter `k` is declared `i64` here",
        "its parameters are `w`, `h`",
    ] {
        assert!(out.stderr.contains(label), "[DIA-22] no secondary span {label:?}:
{}", out.stderr);
    }
    assert!(!out.stderr.contains("needs a value for"), "[DIA-22] one diagnostic per wrong call:
{}", out.stderr);
}

#[test]
fn formatter_keeps_where_clauses() {
    // D-643, `[FMT-1]` — a `where` clause is part of a declaration: the formatter wrote a function's,
    // a type's, an interface's and an extension's header without it, which changed what the
    // program means (`parse(fmt(x))` differed from `parse(x)` only in what the tree did not show).
    let root = workspace_root();
    let directory = temporary_directory("fmt-where");
    let scratch = directory.join(ember_branding::source_file("main"));
    std::fs::write(&scratch, "interface Show:\n    fn show(self) -> int\n\nfn f[T](x: T) -> int where T: Show:\n    return x.show()\n\nstruct W[T]:\n    v: T\n\nextend[T] W[T] implements Show where T: Show:\n    fn show(self) -> int:\n        return self.v.show()\n\nfn main():\n    pass\n").expect("the fixture is writable");
    let path = scratch.to_string_lossy().into_owned();
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    for header in ["fn f[T](x: T) -> int where T: Show:", "extend[T] W[T] implements Show where T: Show:"] {
        assert!(once.stdout.contains(header), "[FMT-1] the formatter dropped a where clause, wanted {header:?}:\n{}", once.stdout);
    }
    std::fs::write(&scratch, &once.stdout).expect("the formatted fixture is writable");
    let twice = ember(&["fmt", &path], &root);
    assert_eq!(once.stdout, twice.stdout, "[FMT-1] fmt(fmt(x)) differs from fmt(x)");
    let checked = ember(&["check", &path], &root);
    assert_eq!(checked.exit, 0, "D-644 the formatted program does not check:\n{}", checked.stderr);
}

#[test]
fn module_attributes_format_from_the_parsed_tree() {
    let root = workspace_root();
    let directory = temporary_directory("module-attr-fmt");
    let scratch = directory.join(ember_branding::source_file("main"));
    std::fs::write(&scratch,
        "#! language \"0.9.10\"\n#! module overflow(wrap)\n#! threads any\n\nfn add(a: i8, b: i8) -> i8:\n    return a + b\n")
        .expect("module attribute fixture is writable");
    let path = scratch.to_string_lossy();
    let once = ember(&["fmt", &path], &root);
    assert_eq!(once.exit, 0, "{}", once.stderr);
    assert!(once.stdout.contains("#! module overflow(wrap)"));
    std::fs::write(&scratch, &once.stdout).expect("formatted fixture is writable");
    let twice = ember(&["fmt", &path], &root);
    assert_eq!(twice.exit, 0, "{}", twice.stderr);
    assert_eq!(once.stdout, twice.stdout);
    let checked = ember(&["fmt", &path, "--check"], &root);
    assert_eq!(checked.exit, 0, "{}", checked.stderr);
}

#[test]
fn hello_world_builds_and_runs() {
    // Phase 0's exit criterion (Part XX.2).
    let root = workspace_root();
    let out_dir = temporary_directory("hello");
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
        temporary_directory("lt40-interface");
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

/// `[EFF-4]` — effects are part of a function's interface for incremental builds: when an edit to
/// a private body gives an imported function a new effect, the importer's contracts are checked
/// again although its own source is unchanged. `main`'s `@noio` caller of `helper.effectful` is
/// accepted while `effectful` is quiet and refused (`E4041`) once `helper`'s private `quiet`
/// prints, with the interface cache of the first check in place.
#[test]
fn a_callee_gaining_an_effect_rechecks_its_callers_contracts() {
    let workspace = workspace_root();
    let test_root = temporary_directory("eff4-contracts");
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    std::fs::create_dir_all(&test_root).expect("create the EFF-4 package");
    let helper_source = |quiet: &str| format!("fn quiet() -> int:\n{quiet}\n\npub fn effectful() -> int:\n    return quiet()\n");
    std::fs::write(test_root.join(&helper), helper_source("    return 1")).expect("write helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import effectful\n\n@noio\nfn steady() -> int:\n    return effectful()\n\nfn main():\n    println(steady())\n",
    )
    .expect("write importer");
    let check = || {
        Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for EFF-4")
    };
    let quiet = check();
    assert!(quiet.status.success(), "the quiet callee is refused:\n{}", String::from_utf8_lossy(&quiet.stderr));
    let before = cached_interface(&out_dir, "helper");
    let effectful = &before.callables[&ember_branding::mangled("helper.effectful")];
    assert!(!effectful.effects.as_ref().expect("a body's effects").iter().any(|effect| effect == "Io"), "{effectful:?}");

    std::fs::write(test_root.join(&helper), helper_source("    println(\"loud\")\n    return 1")).expect("edit helper");
    let loud = check();
    let stderr = String::from_utf8_lossy(&loud.stderr);
    assert!(!loud.status.success() && stderr.contains("E4041") && stderr.contains("steady"),
        "the caller's `@noio` was not checked again:\n{stderr}");
}

/// `[BLD-2]` / `[CG-C-3b]` — a module's interface hash holds the facts a
/// caller may use beside signatures: each callable's effect set, the body the
/// inline header exports, and each visible type's layout. Editing a private
/// body so that a public function gains an effect, editing an exported inline
/// body, or adding a field to a public struct changes the hash and the
/// importer's cache key; an edit that changes none of them (a private
/// function's constant, a private struct) changes neither.
#[test]
fn effects_inline_bodies_and_layouts_are_in_the_interface_hash() {
    let workspace = workspace_root();
    let test_root = temporary_directory("bld2-facts");
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create BLD-2 package");
    let helper_source = |quiet: &str, small: &str, point: &str, private: &str| {
        format!(
            "pub struct Point:\n{point}\n\nstruct Hidden:\n{private}\n\nfn quiet() -> int:\n{quiet}\n\npub fn effectful() -> int:\n    return quiet()\n\npub fn small(x: int) -> int:\n{small}\n"
        )
    };
    let quiet = "    return 1";
    let loud = "    println(\"loud\")\n    return 1";
    let small = "    return x + 1";
    let point = "    pub x: int";
    let private = "    a: int";
    std::fs::write(test_root.join(&helper), helper_source(quiet, small, point, private)).expect("write helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import Point, effectful, small\n\nfn main():\n    p = Point(x=1)\n    println(effectful() + small(p.x))\n",
    )
    .expect("write importer");
    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for BLD-2");
        assert!(output.status.success(), "{label} check failed:\n{}", String::from_utf8_lossy(&output.stderr));
        (cached_interface(&out_dir, "helper"), cached_interface(&out_dir, "root"))
    };
    let (helper0, root0) = check("initial");
    let effectful = &helper0.callables[&ember_branding::mangled("helper.effectful")];
    assert!(!effectful.effects.as_ref().expect("a body's effects").iter().any(|e| e == "Io"), "{effectful:?}");
    assert!(helper0.callables[&ember_branding::mangled("helper.small")].inline_body.is_some());
    assert!(helper0.layouts.keys().any(|name| name.ends_with("Point")), "{:?}", helper0.layouts);
    assert!(!helper0.layouts.keys().any(|name| name.ends_with("Hidden")), "{:?}", helper0.layouts);

    // No fact changes: a private function's constant, a private struct.
    std::fs::write(test_root.join(&helper), helper_source("    return 2", small, point, "    a: int\n    b: int"))
        .expect("edit private items");
    let (helper1, root1) = check("after private edits");
    assert_eq!(helper0.interface_hash, helper1.interface_hash);
    assert_eq!(root0.cache_key, root1.cache_key);

    // A private body gains `IO`, and so does the public function calling it.
    std::fs::write(test_root.join(&helper), helper_source(loud, small, point, private)).expect("make quiet loud");
    let (helper2, root2) = check("after an effect change");
    let effectful = &helper2.callables[&ember_branding::mangled("helper.effectful")];
    assert!(effectful.effects.as_ref().expect("a body's effects").iter().any(|e| e == "Io"), "{effectful:?}");
    assert_ne!(helper1.interface_hash, helper2.interface_hash);
    assert_ne!(root1.cache_key, root2.cache_key);

    // An exported inline body changes; its signature and effects do not.
    std::fs::write(test_root.join(&helper), helper_source(loud, "    return x + 2", point, private)).expect("edit small");
    let (helper3, root3) = check("after an inline body change");
    let symbol = ember_branding::mangled("helper.small");
    assert_eq!(helper2.callables[&symbol].signature, helper3.callables[&symbol].signature);
    assert_eq!(helper2.callables[&symbol].effects, helper3.callables[&symbol].effects);
    assert_ne!(helper2.callables[&symbol].inline_body, helper3.callables[&symbol].inline_body);
    assert_ne!(helper2.interface_hash, helper3.interface_hash);
    assert_ne!(root2.cache_key, root3.cache_key);

    // A public struct gains a field.
    std::fs::write(
        test_root.join(&helper),
        helper_source(loud, "    return x + 2", "    pub x: int\n    pub y: int = 0", private),
    )
    .expect("grow Point");
    let (helper4, root4) = check("after a layout change");
    assert_ne!(helper3.layouts, helper4.layouts);
    assert_ne!(helper3.interface_hash, helper4.interface_hash);
    assert_ne!(root3.cache_key, root4.cache_key);
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[TYP-8]` / ODR-084 — an imported declaration keeps its module's lexical
/// policy. Editing only that directive changes the callable interface and
/// invalidates the importing module even when its own source is unchanged.
#[test]
fn imported_module_overflow_policy_invalidates_callers_interface_key() {
    let workspace = workspace_root();
    let test_root = temporary_directory("overflow-interface");
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create overflow-interface package");
    let helper_source = |policy: &str| format!(
        "#! module overflow({policy})\n\npub fn defaulted(value: i8, next: i8 = value + 1i8) -> i8:\n    return next\n\npub fn generic[T](tag: T, value: i8) -> i8:\n    return value + 1i8\n"
    );
    std::fs::write(test_root.join(&helper), helper_source("wrap"))
        .expect("write initial helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import defaulted, generic\n\nfn main():\n    println(defaulted(1i8), generic[int](0, 1i8))\n",
    )
    .expect("write importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for overflow interfaces");
        assert!(output.status.success(),
            "{label} check failed:\n{}", String::from_utf8_lossy(&output.stderr));
    };

    check("initial");
    let before_helper = cached_interface(&out_dir, "helper");
    let before_root = cached_interface(&out_dir, "root");
    assert!(before_helper.callables.values().all(|contract| contract.signature.overflow == "wrap"));

    std::fs::write(test_root.join(&helper), helper_source("saturate"))
        .expect("change only module policy");
    check("after module policy change");
    let after_helper = cached_interface(&out_dir, "helper");
    let after_root = cached_interface(&out_dir, "root");
    assert!(after_helper.callables.values().all(|contract| contract.signature.overflow == "saturate"));
    assert_ne!(before_helper.interface_hash, after_helper.interface_hash);
    assert_ne!(before_root.cache_key, after_root.cache_key);
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[FN-1]` / `[FFI-5]` / `[BLD-2]` — the cache records declaration modes,
/// rather than MIR's implementation `ref mut` spelling, and retains the ABI
/// and unsafe call boundary as caller-visible signature facts.
#[test]
fn callable_signature_modes_unsafe_and_abi_cross_the_interface_boundary() {
    let workspace = workspace_root();
    let test_root = temporary_directory("callable-signature");
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
    let test_root = temporary_directory("generic-interface");
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
    let test_root = temporary_directory("callable-mode-interface");
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
        "from helper import apply\n\nfn increment(mut value: i32) -> i32:\n    value = value + 1\n    return value\n\nfn main():\n    value: i32 = 4\n    println(apply(increment, value))\n",
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
        "from helper import apply\n\nfn identity(value: i32) -> i32:\n    return value\n\nfn main():\n    value: i32 = 4\n    println(apply(identity, value))\n",
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
    let test_root = temporary_directory("member-interface");
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

/// `[TYP-16]` / `[MOD-2]` / `[BLD-2]` — a public generic-enum member is a
/// source declaration even when no concrete enum instantiation is emitted.
#[test]
fn visible_generic_enum_member_declarations_invalidate_importers() {
    let workspace = workspace_root();
    let test_root = temporary_directory("generic-enum-interface");
    let out_dir = test_root.join("target");
    let helper = ember_branding::source_file("helper");
    let main = ember_branding::source_file("main");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create generic-enum interface package");
    std::fs::write(
        test_root.join(&helper),
        "from std.core import Eq\n\npub enum Message[T: Eq]:\n    Value(value: T)\n\n    pub fn mirror[U: Eq](self, value: U) -> U:\n        return value\n",
    )
    .expect("write initial generic-enum helper");
    std::fs::write(
        test_root.join(&main),
        "from helper import Message\n\nfn main():\n    println(1)\n",
    )
    .expect("write generic-enum importer");

    let check = |label: &str| {
        let output = Command::new(EMBER)
            .args(["check", &main, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs for generic-enum interfaces");
        assert!(
            output.status.success(),
            "{label} generic-enum interface check failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    check("initial");
    let before_helper = cached_interface(&out_dir, "helper");
    let before_root = cached_interface(&out_dir, "root");
    let message = before_helper
        .callables
        .values()
        .find(|contract| {
            contract.metadata.is_none()
                && contract.signature.parameters.first().is_some_and(|parameter| {
                    parameter.ty == "helper.Message[$P0]"
                        && parameter.mode == CallableParameterMode::Borrow
                })
        })
        .expect("generic-enum member declaration is serialized");
    assert_eq!(message.signature.generics.len(), 2);
    let initial_owner_bounds = message.signature.generics[0].bounds.clone();
    assert_eq!(initial_owner_bounds, message.signature.generics[1].bounds);

    std::fs::write(
        test_root.join(&helper),
        "from std.core import Eq\nfrom std.collections import Hash\n\npub enum Message[T: Hash]:\n    Value(value: T)\n\n    pub fn mirror[U: Eq](self, value: U) -> U:\n        return value\n",
    )
    .expect("change generic-enum owner bound");
    check("after generic-enum owner bound change");
    let after_helper = cached_interface(&out_dir, "helper");
    let after_root = cached_interface(&out_dir, "root");
    assert_ne!(before_helper.interface_hash, after_helper.interface_hash);
    assert_ne!(before_root.cache_key, after_root.cache_key);
    let message = after_helper
        .callables
        .values()
        .find(|contract| {
            contract.signature.parameters.first().is_some_and(|parameter| {
                parameter.ty == "helper.Message[$P0]"
                    && parameter.mode == CallableParameterMode::Borrow
            })
        })
        .expect("changed generic-enum declaration remains serialized");
    assert_ne!(initial_owner_bounds, message.signature.generics[0].bounds);
    let _ = std::fs::remove_dir_all(&test_root);
}

/// D-391 / `[LT-40]` — two programs in one folder share its interface cache,
/// so a standard module's record must not depend on which program wrote it.
/// A generic type's method bound over the type's parameter (`Q: Keyed[K]`)
/// once took a program-wide number, which shifted `std.collections`' own
/// `ToKey[K]` bounds: the second program found the first one's record "stale" and
/// stopped with an internal error.
#[test]
fn programs_sharing_a_folder_share_its_interface_cache() {
    let workspace = workspace_root();
    let test_root = temporary_directory("shared-interface-cache");
    let out_dir = test_root.join("target");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create shared-cache folder");
    let plain = ember_branding::source_file("plain");
    let bounded = ember_branding::source_file("bounded");
    std::fs::write(test_root.join(&plain), "fn main():
    println(1)
").expect("write plain program");
    std::fs::write(
        test_root.join(&bounded),
        "interface Keyed[K]:
    fn make_key(self) -> K

struct Keys[K]:
    keys: Array[K] = []

    fn has[Q: Keyed[K]](self, q: Q) -> bool:
        return false

fn main():
    println(2)
",
    )
    .expect("write bounded program");
    for program in [&plain, &bounded, &plain] {
        let output = Command::new(EMBER)
            .args(["check", program, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs");
        assert!(
            output.status.success(),
            "{program} failed after another program filled the folder's cache:
{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let _ = std::fs::remove_dir_all(&test_root);
}

/// `[ECS-7]`, `[LT-40]` — `std.ecs.components()` is written from each program's components, so
/// `std.ecs`'s interface-cache key takes them: two programs with different components in one
/// folder found each other's record "stale" and stopped with an internal error.
#[test]
fn programs_with_different_components_share_a_folder() {
    let workspace = workspace_root();
    let test_root = temporary_directory("component-interface-cache");
    let out_dir = test_root.join("target");
    let _ = std::fs::remove_dir_all(&test_root);
    std::fs::create_dir_all(&test_root).expect("create shared-cache folder");
    let one = ember_branding::source_file("one");
    let two = ember_branding::source_file("two");
    let program = |extra: &str| format!("from std.ecs import World, Component

@derive(Component)
struct A:
    x: int
{extra}
fn main():
    w = World()
    e = w.create()
    w.add(e, A(1))
");
    std::fs::write(test_root.join(&one), program("")).expect("write the first program");
    std::fs::write(test_root.join(&two), program("
@derive(Component)
struct B:
    y: int
")).expect("write the second program");
    for program in [&one, &two, &one] {
        let output = Command::new(EMBER)
            .args(["check", program, "--out-dir", &out_dir.to_string_lossy()])
            .current_dir(&test_root)
            .env(ember_branding::std_path_var(), workspace.join("std"))
            .output()
            .expect("the Ember compiler runs");
        assert!(
            output.status.success(),
            "{program} failed after another program filled the folder's cache:
{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
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
#$ profiles: debug, development, release
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
    assert_eq!(parsed.profiles, ["debug", "development", "release"]);
    assert_eq!(parsed.assert_c, vec![(false, alloc)]);
    assert_eq!(parsed.assert_c_count, vec![(arena_alloc, 1)]);
    assert_eq!(parsed.helps, ["keep one owner"]);
    assert_eq!(parsed.forbidden_helps, ["RefCell"]);

    // D-184 — the bare form names a code and requires it.
    let bare = parse_expectations("#$ test: compile-fail\n#$ error[E3062]\n#$ error[E2020]: expected\n");
    assert_eq!(
        bare.errors,
        [("E3062".to_string(), String::new()), ("E2020".to_string(), "expected".to_string())]
    );
}

/// D-272 — the generated C touches a 128-bit value only through the
/// runtime's helpers, so a program means the same where the runtime carries
/// `i128` and `u128` as two 64-bit halves (MSVC's form). Here the halves are
/// forced on a compiler that has `__int128`, where a C operator applied to one
/// would otherwise pass unnoticed: it must compile without a warning and print
/// what the native build prints.
#[test]
fn the_128_bit_programs_run_with_the_halves() {
    let root = workspace_root();
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).expect("a C toolchain");
    let (ember_build::Toolchain::Clang(cc) | ember_build::Toolchain::Gcc(cc)) = &toolchain else { return };
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let dir = temporary_directory("int128-halves");
    for relative in [
        "tests/conformance/TYP-1/accept_128_bit_integers",
        "tests/conformance/TYP-6/accept_128_bit_casts",
        "tests/conformance/TXT-10/accept_parse_128_bit_integers",
        "tests/conformance/STD-26/accept_ranges_of_128_bit_integers",
        "tests/conformance/STD-11/accept_128_bit_keys",
        "tests/conformance/STD-20/accept_integer_methods",
        // D-421 — a range type over a 128-bit integer.
        "tests/conformance/RNG-1/accept_a_range_type_over_128_bit_integers",
    ] {
        let relative = format!("{relative}.{SOURCE_EXT}");
        let native = ember(&["run", &relative, "--out-dir", &dir.join("native").to_string_lossy()], &root);
        assert_eq!(native.exit, 0, "{relative}: {}", native.stderr);
        let emitted = ember(&["build", &relative, "--emit", "c"], &root);
        assert_eq!(emitted.exit, 0, "emitting C for {relative} failed:\n{}", emitted.stderr);
        let source = dir.join("program.c");
        std::fs::write(&source, &emitted.stdout).expect("the C is writable");
        let program = dir.join("program");
        let mut compile = Command::new(cc);
        compile
            .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-O1"])
            .arg(format!("-D{}_SOFT_INT128", ember_branding::SYMBOL_PREFIX.to_uppercase()))
            .arg("-I")
            .arg(runtime.join("include"))
            .arg(&source)
            .arg(runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX)))
            .arg("-o")
            .arg(&program);
        if !cfg!(windows) {
            compile.arg("-lm");
        }
        let compiled = compile.output().expect("the C compiler runs");
        assert!(
            compiled.status.success(),
            "{relative} with the halves:\n{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let ran = ember_build::output_started(&mut Command::new(&program)).expect("the program runs");
        assert_eq!(String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n"), native.stdout, "{relative}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[TST-4]`/`[TST-4a]` — every rule directory under `tests/conformance/` is
/// run, and every file in one carries its expectations. A directory that
/// exists but is never executed is what `[TST-4a]` calls "not coverage".
/// `[TST-27]` — each accepted program's C is compiled with warnings as errors (`ember_gated`), on
/// each C compiler CI runs this suite with.
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

    // Every directory's own checks first; then all the cases in one run.
    let mut cases = Vec::new();
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
        }
        cases.extend(files);
    }
    // CI splits the cases over machines: `EMBER_TEST_SHARD=i/n` keeps every
    // n-th case from the i-th, so n jobs between them run every case.
    if let Ok(shard) = std::env::var("EMBER_TEST_SHARD")
        && !shard.is_empty()
    {
        let (index, count) = shard
            .split_once('/')
            .and_then(|(i, n)| Some((i.parse::<usize>().ok()?, n.parse::<usize>().ok()?)))
            .filter(|&(i, n)| (1..=n).contains(&i))
            .unwrap_or_else(|| panic!("EMBER_TEST_SHARD={shard}: expected i/n with 1 <= i <= n"));
        cases = cases.into_iter().skip(index - 1).step_by(count).collect();
        println!("conformance share {index}/{count}: {} cases", cases.len());
    }
    report_failures("tests/conformance", &run_cases(&cases, |path| check_file(path, &root)));
}

/// The definition of the generated C function `name` (from its header line
/// to its closing brace), for a test reading what one function became.
fn c_definition<'a>(c: &'a str, name: &str) -> &'a str {
    let call = format!("{name}(");
    let start = c
        .match_indices(&call)
        .map(|(at, _)| at)
        .find(|&at| c[at..].split('\n').next().is_some_and(|line| line.ends_with('{')))
        .unwrap_or_else(|| panic!("{name} is defined:\n{c}"));
    let start = c[..start].rfind('\n').map_or(0, |at| at + 1);
    let end = start + c[start..].find("\n}\n").expect("the function ends");
    &c[start..end]
}

/// ADR-106 — `s.chars()`, `s.char_indices()` and `s.bytes()` in a `for`
/// header are counted loops with no iterator: `main` decodes with the text
/// loop's step and calls neither `next`. An iterator in a variable still
/// calls `next` with clang (with MSVC its step is copied in, ADR-107).
#[test]
fn text_iterator_loops_are_counted_loops_without_the_iterator() {
    let root = workspace_root();
    let source = format!("tests/conformance/TXT-10/accept_text_iterators_in_loops_and_through_next.{SOURCE_EXT}");
    for cc in ["msvc", "clang"] {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc} C failed:\n{}", c.stderr);
        let main = c_definition(&c.stdout, &ember_branding::mangled("main"));
        // ADR-155: MSVC's native step behind the ASCII test takes the lead byte.
        let native = main.contains("str_char_next_usize(") || main.contains("str_char_next_wide_usize(");
        assert!(native, "{cc}: no native text loop in main:\n{main}");
        assert!(!main.contains("CharIndices_next(") && !main.contains("Bytes_next("), "{cc}: a loop calls `next`:\n{main}");
        if cc == "clang" {
            assert!(main.contains("Chars_next("), "clang: the iterator held in a variable lost its `next`:\n{main}");
        }
    }
}

/// ADR-106 — a struct, tuple or payload variant assigned to a local is
/// written field by field, the tag first, and a unit variant writes only its
/// tag; a view a built-in makes and an `Option` of a view's `None` take
/// their two fields one at a time. As compound literals MSVC built them in
/// memory and read them back wider than they were written (`char_indices()`
/// 15x the C loop). D-475 — a view copied whole from a place its function
/// writes by fields is copied by its fields too (`lines()` 1.98x the C with
/// MSVC copied whole), and from a place written whole it stays one
/// assignment (by its fields, MSVC no longer interchanged `a05_structs`'s
/// loop nest: 1.47x).
#[test]
fn aggregates_and_views_are_written_field_by_field() {
    let root = workspace_root();
    let source = format!("tests/conformance/TXT-10/accept_text_iterators_in_loops_and_through_next.{SOURCE_EXT}");
    // clang's C: with MSVC these steps are copied into their loops (ADR-107);
    // the writes by fields are every compiler's.
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(c.exit, 0, "clang C failed:\n{}", c.stderr);
    let next = c_definition(&c.stdout, &format!("{}_CharIndices_next", ember_branding::mangled("std_string")));
    // ADR-135 (G7-2): the item's tuple is a literal written in place, its own
    // fields under the payload's.
    assert!(
        next.contains("(_0).tag = 1;") && next.contains("_0.payload.Some._0._0 = ") && next.contains("_0.payload.Some._0._1 = "),
        "the item is not written by fields:\n{next}"
    );
    assert!(next.contains("(_0).tag = 0;") && !next.contains("memset(") && !next.contains(".tag = 1, .payload"), "`None` or `Some` is a literal:\n{next}");
    let lines = c_definition(&c.stdout, &format!("{}_Lines_next", ember_branding::mangled("std_string")));
    assert!(lines.contains("((*_1).rest).ptr = ") && lines.contains("((*_1).rest).len = "), "the slice is a literal:\n{lines}");
    assert!(lines.contains("(_0).ptr = NULL;") && !lines.contains("{ .ptr = NULL"), "`None` of a view is a literal:\n{lines}");
    assert!(lines.contains("(_2).ptr = ((*_1).rest).ptr;"), "the rest, written by fields, is copied whole:\n{lines}");
    let counted = format!("tests/conformance/CTL-1/accept_a_view_iterator_loop_is_counted.{SOURCE_EXT}");
    let c = ember(&["build", &counted, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(c.exit, 0, "clang C failed:\n{}", c.stderr);
    assert!(c.stdout.contains(".source;") && !c.stdout.contains(".source).ptr;"), "an iterator's view, written whole, is copied by its fields:\n{}", c.stdout);
    // D-501 — only a source read from memory is copied by its fields: a local
    // whose address is never taken lives in registers and is copied whole.
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(c.exit, 0, "clang C failed:\n{}", c.stderr);
    let words = c_definition(&c.stdout, &format!("{}_SplitWhitespace_next", ember_branding::mangled("std_string")));
    assert!(words.contains("(_2).ptr = ((*_1).rest).ptr;"), "the rest, read from memory, is copied whole:\n{words}");
    assert_eq!(register_field_copies(words), 0, "a register local is copied by its fields:\n{words}");
}

/// The view copies `(_a).ptr = (_b).ptr;` whose source is a plain local.
fn register_field_copies(c: &str) -> usize {
    c.lines()
        .map(str::trim)
        .filter(|line| {
            line.strip_prefix("(_").and_then(|rest| rest.split_once(").ptr = (_")).is_some_and(|(_, source)| {
                source.strip_suffix(").ptr;").is_some_and(|local| !local.is_empty() && local.chars().all(|c| c.is_ascii_digit()))
            })
        })
        .count()
}

/// A nested field producer obeys D-475 through niche Option wrapping and
/// extraction: the split payload, read from memory, is copied by its fields,
/// pointer and length together (a register local's copies are whole, D-501,
/// tested on `split_whitespace` above). The existing aggregate test above
/// retains the inverse proof: a wholly written iterator view stays copied
/// whole.
#[test]
fn split_once_view_fields_stay_field_copies_through_niche_options() {
    fn payload_copied_by_fields(c: &str, source_suffix: &str) {
        let copies: Vec<&str> = c.lines().map(str::trim).filter(|line| line.contains(".ptr = ") && line.ends_with(source_suffix)).collect();
        assert!(!copies.is_empty(), "the split payload is read whole:\n{c}");
        for copy in copies {
            let (to, from) = copy.split_once(".ptr = ").expect("a pointer field copy");
            let from = from.strip_suffix(".ptr;").expect("a pointer field copy");
            let length = format!("{to}.len = {from}.len;");
            assert!(c.lines().any(|line| line.trim() == length), "the descriptor's length is not copied with its pointer:\n{c}");
        }
    }
    let root = workspace_root();
    let source = format!("tests/conformance/TXT-10/accept_split_once_view_copy_chains.{SOURCE_EXT}");
    for cc in ["msvc", "clang"] {
        let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc} C failed:\n{}", c.stderr);
        let first = c_definition(&c.stdout, &ember_branding::mangled("first_view"));
        assert!(first.contains(").tag = 0;") && !first.contains("{ .tag = 0 }"), "split None is not tag-only:\n{first}");
        payload_copied_by_fields(first, ".payload.Some._0._0).ptr;");
        payload_copied_by_fields(first, ".payload.Some._0._1).ptr;");
    }
}

/// `[CG-C-3]` — a standard view's or container's `next` is forced inline,
/// and a function of at most 40 statements is `static inline`; `[CG-C-3a]`
/// — `@inline` is forced, `@noinline`, `@cold` and `@hot` reach the C
/// compiler, and a `@noinline` function called once stays a call even for
/// MSVC, where the MIR inliner takes every other such function.
#[test]
fn inline_header_functions_and_inline_attributes_reach_the_c() {
    let root = workspace_root();
    let iterators = format!("tests/conformance/TXT-10/accept_text_iterators_in_loops_and_through_next.{SOURCE_EXT}");
    // clang's C keeps `Chars.next` a function (with MSVC it is copied into
    // its loops, ADR-107); how it is marked is every compiler's.
    let c = ember(&["build", &iterators, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(c.exit, 0, "clang C failed:\n{}", c.stderr);
    let std_string = ember_branding::mangled("std_string");
    assert!(c.stdout.contains(&format!("EMBER_INLINED EMBER_MAYBE_UNUSED {} {std_string}_Chars_next(", ember_branding::mangled("Option_char"))), "`Chars.next` is not forced inline:\n{}", c.stdout);
    assert!(c.stdout.contains(&format!("static inline EMBER_MAYBE_UNUSED {std_string}_Chars {}(", ember_branding::mangled("str_chars"))), "`str.chars` is not static inline:\n{}", c.stdout);
    let attributes = format!("tests/conformance/CG-C-3a/accept_inline_noinline_cold_and_hot.{SOURCE_EXT}");
    for cc in ["msvc", "clang"] {
        let c = ember(&["build", &attributes, "--emit", "c", "--profile", "development", "--cc", cc], &root);
        assert_eq!(c.exit, 0, "{cc} C failed:\n{}", c.stderr);
        let header = |name: &str| {
            let call = format!("{}(", ember_branding::mangled(name));
            c.stdout.lines().find(|line| line.contains(&call) && line.ends_with('{')).map(str::to_string).unwrap_or_default()
        };
        assert!(header("twice").starts_with("EMBER_INLINED "), "{cc}: `@inline` is not forced: {}", header("twice"));
        assert!(header("thrice").starts_with("EMBER_NOINLINE "), "{cc}: `@noinline` is lost: {}", header("thrice"));
        assert!(header("failed").starts_with("EMBER_COLD "), "{cc}: `@cold` is lost: {}", header("failed"));
        assert!(header("step").starts_with("EMBER_HOT "), "{cc}: `@hot` is lost: {}", header("step"));
        let main = c_definition(&c.stdout, &ember_branding::mangled("main"));
        assert!(main.contains(&format!("{}(", ember_branding::mangled("thrice"))), "{cc}: the `@noinline` call was inlined:\n{main}");
    }
}

/// ADR-106 — a counted loop pushing a literal onto a `String` on every turn
/// has room for all its bytes asked for before it, as for `Array.push`.
#[test]
fn a_loop_pushing_a_literal_reserves_its_text() {
    let root = workspace_root();
    let source = format!("tests/conformance/TXT-11/accept_a_string_has_every_str_method.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(c.exit, 0, "clang C failed:\n{}", c.stderr);
    let main = c_definition(&c.stdout, &ember_branding::mangled("main"));
    assert!(main.contains("vec_reserve_hint(") && main.contains(", 2u)"), "no room is reserved for the pushes:\n{main}");
}

/// ADR-106 — a `for` over an iterator without an `else` leaves by `break`
/// when `next` gives `None`: no `__done` flag tested on every turn.
#[test]
fn an_iterator_loop_without_else_has_no_flag() {
    let root = workspace_root();
    let source = format!("tests/conformance/TYP-17/accept_an_inherent_next_is_the_iterators_for_a_bound_call.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(c.exit, 0, "msvc C failed:\n{}", c.stderr);
    let main = c_definition(&c.stdout, &ember_branding::mangled("main"));
    assert!(main.contains("Countdown_next(") && !main.contains("__done"), "the loop keeps a flag:\n{main}");
}

/// ADR-107 — for MSVC, a `for` loop's step through a standard iterator is
/// copied into the loop, and the variant it builds goes straight to its
/// branch: the loop calls no `next`, and no test of what came back is ever
/// entered (its block keeps no label). clang's C keeps the call.
#[test]
fn msvc_loops_copy_a_standard_step_and_skip_its_answer() {
    let root = workspace_root();
    let source = format!("tests/conformance/TXT-10/accept_text_iterators_in_loops_and_through_next.{SOURCE_EXT}");
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:\n{}", msvc.stderr);
    let main = c_definition(&msvc.stdout, &ember_branding::mangled("main"));
    for step in ["Lines_next(", "Split_next(", "SplitWhitespace_next("] {
        assert!(!main.contains(step), "the loop still calls {step}:\n{main}");
    }
    // `#line` markers sit between a label and its first statement.
    let lines: Vec<&str> = main.lines().filter(|line| !line.starts_with("#line")).collect();
    let entered_test = lines.windows(2).any(|pair| {
        pair[0].starts_with("bb") && pair[0].ends_with(": ;") && pair[1].contains(".ptr == NULL && (") && pair[1].contains(".len == SIZE_MAX ? 0 : 1);")
    });
    assert!(!entered_test, "a loop still tests what its step gave back:\n{main}");
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "development", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:\n{}", clang.stderr);
    assert!(c_definition(&clang.stdout, &ember_branding::mangled("main")).contains("Split_next("), "clang's loop lost its call");
}

/// D-474 — a body instantiated from `std` (a generic function's instance, a
/// generic type's method, an interface's provided method) is kept only if
/// something reaches it: a program that names no text iterator checks none
/// of `Iterator`'s provided methods for std's six, nor the adapters those
/// name (every program checked about 150 of them, 4x the time), and one that
/// calls `count` keeps that one.
#[test]
fn std_instances_nothing_reaches_are_not_kept() {
    let root = workspace_root();
    let std_string = ember_branding::mangled("std_string");
    let unused = [
        format!("fn {std_string}_Chars_count:"),
        format!("fn {std_string}_Split_step_by:"),
        format!("fn {}_Enumerate_std_string_Lines_next:", ember_branding::mangled("std_core")),
    ];
    let plain = format!("tests/milestones/m1_value_code_has_no_runtime_cost.{SOURCE_EXT}");
    let mir = ember(&["build", &plain, "--emit", "mir", "--profile", "development"], &root);
    assert_eq!(mir.exit, 0, "MIR failed:\n{}", mir.stderr);
    for body in &unused {
        assert!(!mir.stdout.contains(body.as_str()), "`{body}` is kept though nothing reaches it");
    }
    let iterators = format!("tests/conformance/TXT-10/accept_text_iterators_in_loops_and_through_next.{SOURCE_EXT}");
    let mir = ember(&["build", &iterators, "--emit", "mir", "--profile", "development"], &root);
    assert_eq!(mir.exit, 0, "MIR failed:\n{}", mir.stderr);
    assert!(mir.stdout.contains(unused[0].as_str()), "`count`, which the program calls, was dropped");
}

// D-491: verify the cache before indexing without executing broken C.
fn assert_memreplace_span_result_pointer_is_refreshed(c: &str) {
    let span = ember_branding::runtime("span");
    let load_marker = format!(" = *(({span}*)(");
    let lines: Vec<_> = c.lines().map(str::trim).collect();
    let (load_at, destination, target) = lines.iter().enumerate().find_map(|(index, line)| {
        let (destination, target) = line.split_once(&load_marker)?;
        let target = target.strip_suffix("));")?;
        Some((index, destination.to_string(), target.to_string()))
    }).unwrap_or_else(|| panic!("no direct Span MemReplace result load:\n{c}"));
    let replacement_prefix = format!("*(({span}*)({target})) = ");
    let replacement_at = lines.iter().enumerate().skip(load_at + 1)
        .find_map(|(index, line)| line.starts_with(&replacement_prefix).then_some(index))
        .unwrap_or_else(|| panic!("MemReplace does not store its replacement:\n{c}"));
    let pointer = format!("{destination}_ptr");
    let index_marker = format!("{pointer}[");
    let first_index_at = lines.iter().position(|line| line.contains(&index_marker))
        .unwrap_or_else(|| panic!("the result has no dynamic cached-pointer index:\n{c}"));
    let refresh_prefix = format!("{pointer} = ");
    let refresh_suffix = format!("{destination}.ptr;");
    let refresh_at = lines.iter().position(|line|
        line.starts_with(&refresh_prefix) && line.ends_with(&refresh_suffix))
        .unwrap_or_else(|| panic!("MemReplace leaves its result pointer cache uninitialized:\n{c}"));
    assert!(load_at < replacement_at && replacement_at < refresh_at && refresh_at < first_index_at,
        "the result pointer cache is not refreshed after the exchange and before indexing:\n{c}");
    // The addressed target cannot retain a duplicate pointer that the exchange
    // silently invalidates. If it renders as a direct local address, pin that
    // inverse policy without depending on MIR local numbers.
    if let Some(addressed) = target.strip_prefix('&') {
        let addressed = addressed.trim_matches(|ch| ch == '(' || ch == ')');
        assert!(!c.contains(&format!("{addressed}_ptr")),
            "the addressed replacement target retained a pointer cache:\n{c}");
    }
}

#[test]
fn memreplace_span_results_refresh_their_index_pointer() {
    let root = workspace_root();
    let source = format!("tests/conformance/OWN-6/accept_span_replace_result_refreshes_index_pointer.{SOURCE_EXT}");
    for profile in ["debug", "development", "release"] {
        for cc in ["msvc", "clang"] {
            let c = ember(&["build", &source, "--emit", "c", "--profile", profile, "--cc", cc], &root);
            assert_eq!(c.exit, 0, "{cc}/{profile} C generation failed:\n{}", c.stderr);
            let replaced = c_definition(&c.stdout, &ember_branding::mangled("replace_shared_view"));
            assert_memreplace_span_result_pointer_is_refreshed(replaced);
        }
    }
}

/// Semantic progress can remove a per-character add check only behind a
/// verified entry guard. Both the optimized path and checked fallback reach C;
/// an effectful loop remains checked. The native annotations independently
/// check Unicode/NUL offsets, negative starts, empty input and original panic.
#[test]
fn progress_reduction_emits_a_fast_loop_and_preserves_checked_fallback() {
    let root = workspace_root();
    let safe = format!(
        "tests/conformance/RNG-4/accept_progress_reductions_keep_unicode_and_signed_start.{SOURCE_EXT}"
    );
    let effectful = format!(
        "tests/conformance/RNG-4/run_fail_progress_reduction_keeps_effects_before_overflow.{SOURCE_EXT}"
    );
    for cc in ["msvc", "clang", "gcc"] {
        let c = ember(
            &[
                "build",
                &safe,
                "--emit",
                "c",
                "--profile",
                "development",
                "--cc",
                cc,
            ],
            &root,
        );
        assert_eq!(
            c.exit, 0,
            "{cc}: progress loop failed to emit:\n{}",
            c.stderr
        );
        for name in ["reduce", "reduce_skipping_nul"] {
            let body = c_definition(&c.stdout, &ember_branding::mangled(name));
            assert!(
                !body.contains("._1"),
                "{cc}: a text descriptor length was emitted as a tuple field:\n{body}"
            );
            // D-498 (ADR-155) — the fast loop's ASCII step; on MSVC its other
            // bytes reach the decoder entry that takes the tested lead byte.
            // D-550 (ADR-199) — gcc's mixed and short loops decode through it too,
            // its English loop through the plain decoder.
            let (plain, wide) = match cc {
                "msvc" => (1, 1),
                "gcc" => (2, 2),
                _ => (2, 0),
            };
            assert_eq!(
                body.matches("str_char_next_usize(").count(),
                plain,
                "{cc}: guarded loop and checked fallback missing from {name}:\n{body}"
            );
            assert_eq!(
                body.matches("str_char_next_wide_usize(").count(),
                wide,
                "{cc}: the wide decoder entry is MSVC's and gcc's mixed loops' only:\n{body}"
            );
            if cc == "gcc" {
                assert!(body.contains("< 256ULL") && body.contains(">= 32ULL") && body.contains("< 4ULL"),
                    "{cc}: the mixed and short loops' tests are missing from {name}:\n{body}");
            }
            assert!(body.contains("< 128U"), "{cc}: the ASCII step is missing from {name}:\n{body}");
            assert_eq!(
                body.matches("ck_add_i64(").count(),
                1,
                "{cc}: the fallback's original accumulation check changed:\n{body}"
            );
            assert_eq!(
                body.matches("panic_overflow(").count(),
                1,
                "{cc}: the fallback's original panic changed:\n{body}"
            );
            // D-498 — the guard multiplies, checked, and divides nothing.
            assert_eq!(body.matches("ck_mul_u64(").count(), 1, "{cc}: the guard's checked multiply is missing:\n{body}");
            assert!(!body.contains(" / "), "{cc}: the guard divides:\n{body}");
        }
        let c = ember(
            &[
                "build",
                &effectful,
                "--emit",
                "c",
                "--profile",
                "development",
                "--cc",
                cc,
            ],
            &root,
        );
        assert_eq!(
            c.exit, 0,
            "{cc}: effectful loop failed to emit:\n{}",
            c.stderr
        );
        let body = c_definition(&c.stdout, &ember_branding::mangled("effectful"));
        // One decoder call (MSVC's takes the tested lead byte, ADR-155), and one
        // each in gcc's mixed and short loops (ADR-199).
        assert_eq!(
            body.matches("str_char_next_usize(").count() + body.matches("str_char_next_wide_usize(").count(),
            if cc == "gcc" { 3 } else { 1 },
            "{cc}: unsupported effects were included in a fast loop:\n{body}"
        );
        // D-498 — the loop and each ASCII copy (ADR-155: one per byte test,
        // gcc's written-out steps included) keep the check; no add wraps.
        // gcc's short loop gives its wide characters a copy of their own (ADR-199).
        assert_eq!(
            body.matches("ck_add_i64(").count(),
            1 + body.matches("< 128U").count() + usize::from(cc == "gcc"),
            "{cc}: unsupported effectful accumulation lost its check:\n{body}"
        );
        assert!(
            !body.contains(") + (uint64_t)("),
            "{cc}: unsupported effectful accumulation wraps:\n{body}"
        );
        assert!(body.contains("< 128U"), "{cc}: the ASCII step is missing:\n{body}");
    }
}

#[test]
fn string_descriptor_fields_keep_their_source_and_projected_copy_route() {
    // Test-side C shape inspection only. Match both adjacent writes, including
    // identical destination AND source; unrelated field strings cannot pass.
    fn borrowed_pairs(c: &str) -> Vec<(String, String)> {
        let lines: Vec<_> = c.lines().map(str::trim).collect();
        let mut pairs = Vec::new();
        for pair in lines.windows(2) {
            let Some((dest, pointer)) = pair[0].split_once(".ptr = (const unsigned char*)") else {
                continue;
            };
            let Some(source) = pointer.strip_suffix("->ptr;") else {
                continue;
            };
            assert_eq!(
                pair[1],
                format!("{dest}.len = {source}->len;"),
                "unpaired borrowed descriptor:\n{c}"
            );
            pairs.push((dest.to_string(), source.to_string()));
        }
        assert!(
            !pairs.is_empty(),
            "no String descriptor field producer:\n{c}"
        );
        pairs
    }
    fn copy_edges(c: &str) -> Vec<(String, String)> {
        let lines: Vec<_> = c.lines().map(str::trim).collect();
        let mut edges = Vec::new();
        for pair in lines.windows(2) {
            let Some((dest, pointer)) = pair[0].split_once(".ptr = ") else {
                continue;
            };
            let Some(source) = pointer.strip_suffix(".ptr;") else {
                continue;
            };
            assert_eq!(
                pair[1],
                format!("{dest}.len = {source}.len;"),
                "unpaired descriptor copy:\n{c}"
            );
            edges.push((source.to_string(), dest.to_string()));
        }
        edges
    }
    let root = workspace_root();
    let source = format!("tests/conformance/SPN-1/accept_string_descriptor_fields.{SOURCE_EXT}");
    for profile in ["debug", "development", "release"] {
        for cc in ["msvc", "clang", "gcc"] {
            let c = ember(
                &[
                    "build",
                    &source,
                    "--emit",
                    "c",
                    "--profile",
                    profile,
                    "--cc",
                    cc,
                ],
                &root,
            );
            assert_eq!(
                c.exit, 0,
                "{cc}/{profile}: String descriptor emission failed:\n{}",
                c.stderr
            );
            for name in ["borrow_explicit", "borrow_implicit", "descriptor_chain"] {
                let function = c_definition(&c.stdout, &ember_branding::mangled(name));
                assert!(
                    !function.contains("vec_as_str("),
                    "{cc}/{profile}: {name} retains the helper:\n{function}"
                );
                let producers = borrowed_pairs(function);
                assert_eq!(
                    producers.len(),
                    1,
                    "{cc}/{profile}: {name} did not keep its one source conversion:\n{function}"
                );
                let edges = copy_edges(function);
                // D-501: a copy read from memory (a projection) is by fields;
                // a copy of a register local is whole, so the chain to the
                // return need not be fields all the way.
                if name == "descriptor_chain" {
                    assert!(
                        edges.iter().any(|(from, _)| from.contains("._0[")),
                        "{cc}/{profile}: the Option copy read from the array is not by fields:\n{function}"
                    );
                }
            }
        }
    }
}

/// D-500 — the one-byte search lets its found case fall through: written as
/// `hit ? ... : SIZE_MAX`, MSVC turned it into a `cmov` after `memchr` and a
/// taken jump on every call, and `lines()` ran 1.05x.
#[test]
fn one_byte_search_falls_through_when_found() {
    let runtime = workspace_root().join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let header = std::fs::read_to_string(runtime.join("include").join(ember_branding::runtime_header())).expect("the runtime header");
    let text = ember_branding::runtime("str");
    let signature = format!("size_t {}({text} s, {text} needle, size_t from) {{", ember_branding::runtime("str_search"));
    let start = header.find(&signature).expect("the search");
    let search = &header[start..start + header[start..].find("\n}\n").expect("its end")];
    let one_byte = &search[search.find("if (needle.len == 1) {").expect("the one-byte path")..];
    assert!(one_byte.contains("if (from < s.len) {") && one_byte.contains("if (hit != NULL) {"), "the found case does not fall through:\n{search}");
    assert!(!search.contains("return hit ?"), "the answer is a conditional expression:\n{search}");
}

/// `[FN-8]` (D-548) — `args` is the command line decoded as UTF-8, each
/// invalid sequence replaced by U+FFFD: raw bytes on Linux, UTF-16 on Windows
/// (whose C `argv` is in the ANSI code page).
#[test]
fn main_args_replace_text_that_is_not_utf8() {
    let root = workspace_root();
    let out = temporary_directory("main-args");
    let program = format!("tests/conformance/FN-8/accept_main_takes_the_command_line.{SOURCE_EXT}");
    let built = ember(&["build", &program, "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(built.exit, 0, "build failed:\n{}", built.stderr);
    let exe = out.join("debug").join("bin").join(format!(
        "accept_main_takes_the_command_line{}",
        std::env::consts::EXE_SUFFIX
    ));
    #[cfg(unix)]
    let args: Vec<std::ffi::OsString> = {
        use std::os::unix::ffi::OsStrExt;
        [&b"a\xffb"[..], b"\xe2\x82", b"\xf0\x9f\x98\x80", b"\xed\xa0\x80z"]
            .iter()
            .map(|bytes| std::ffi::OsStr::from_bytes(bytes).to_os_string())
            .collect()
    };
    #[cfg(unix)]
    let expected = "5\n[a\u{fffd}b]\n[\u{fffd}]\n[\u{1f600}]\n[\u{fffd}\u{fffd}\u{fffd}z]";
    #[cfg(windows)]
    let args: Vec<std::ffi::OsString> = {
        use std::os::windows::ffi::OsStringExt;
        vec![
            std::ffi::OsString::from_wide(&[0x61, 0xD800, 0x62]),
            std::ffi::OsString::from("\u{1f600}"),
            std::ffi::OsString::from("caf\u{e9}"),
        ]
    };
    #[cfg(windows)]
    let expected = "4\n[a\u{fffd}b]\n[\u{1f600}]\n[caf\u{e9}]";
    let run = ember_build::output_started(Command::new(&exe).args(&args)).expect("the program starts");
    let stdout = String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n");
    // Escaped: a CI runner's log writer fails on an emoji in a failure message.
    assert_eq!(
        stdout.trim_end().escape_default().to_string(),
        expected.escape_default().to_string(),
        "stderr:\n{}",
        String::from_utf8_lossy(&run.stderr).escape_default()
    );
}

/// `[RT-2]` — the runtime has no global constructors: nothing in its C runs before the
/// program's `main`, which starts it by calling `ember_rt_init` (the conformance case
/// `RT-2/accept_main_initialises_the_runtime` checks that half on the emitted C).
#[test]
fn runtime_has_no_global_constructors() {
    let runtime = ember_branding::runtime("rt");
    let templates = workspace_root().join("runtime").join(&runtime).join("templates");
    for name in [format!("{runtime}.c.in"), format!("{}.in", ember_branding::runtime_header())] {
        let text = std::fs::read_to_string(templates.join(&name)).expect("the runtime template reads");
        for needle in ["__attribute__((constructor", "__attribute__ ((constructor", ".CRT$XC", "init_seg"] {
            assert!(!text.contains(needle), "{name} asks for code before `main`: `{needle}`");
        }
    }
}

/// `[VER-1]` — three version numbers, each moving on its own: the language version is the
/// development target's (`#! language` may name only it, `[VER-8]`; D-591 was the compiler still
/// on 0.9.9 under a 0.9.10 document), the compiler's is its package's, and the runtime's ABI
/// version is `EMBER_RUNTIME_ABI`.
#[test]
fn the_three_versions_are_separate() {
    let root = workspace_root();
    let pin = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the pin reads");
    let path = pin.split("\"path\": \"").nth(1).and_then(|rest| rest.split('"').next()).expect("the pin names a document");
    let document = std::fs::read_to_string(root.join(path)).expect("the development target reads");
    let version = document
        .lines()
        .find_map(|line| line.strip_prefix("**Version:** "))
        .expect("the document states its version");
    let language = version.split("_Hardened").next().expect("a version");
    assert_eq!(ember_parser::LANGUAGE_VERSION, language, "the compiler's language is not the document's");

    let run = ember(&["--version"], &root);
    assert_eq!(run.exit, 0);
    assert_eq!(run.stdout.lines().next().unwrap_or(""), format!("{} {}", ember_branding::CLI_NAME, env!("CARGO_PKG_VERSION")));
    assert!(run.stdout.contains(&format!("language: {language}")), "{}", run.stdout);

    let header = std::fs::read_to_string(
        root.join("runtime").join(ember_branding::runtime("rt")).join("templates").join(format!("{}.in", ember_branding::runtime_header())),
    )
    .expect("the runtime header reads");
    let abi = header
        .lines()
        .find_map(|line| line.strip_prefix(&format!("#define {}_RUNTIME_ABI ", ember_branding::SYMBOL_PREFIX.to_uppercase())))
        .expect("the runtime header defines its ABI version");
    assert!(abi.trim().parse::<u32>().is_ok(), "the runtime ABI version is a number: {abi}");
}

/// A package for `ember test` in a directory of its own: its manifest and
/// `src/main` (or `src/lib`) with `source`.
fn test_package(name: &str, kind: &str, source: &str) -> TemporaryDirectory {
    let package = temporary_directory(name);
    std::fs::write(package.join(MANIFEST), format!("[package]\nname = \"{name}\"\nkind = \"{kind}\"\n"))
        .expect("the manifest is writable");
    std::fs::create_dir_all(package.join("src")).expect("src is writable");
    let entry = if kind == "bin" { "main" } else { "lib" };
    std::fs::write(package.join("src").join(ember_branding::source_file(entry)), source).expect("the source is writable");
    package
}

/// `[TST-3]`, `[TST-30]` — `ember test` runs each `@test` function of the
/// package in a process of its own, `@should_panic` passes only by
/// panicking, every failure of the run is reported and not only the first,
/// and the filter selects tests by name. The package's `main` is not run.
#[test]
fn ember_test_runs_each_test_alone_and_reports_every_failure() {
    let package = test_package("tested", "bin", "\
fn add(a: int, b: int) -> int:
    return a + b

@test
fn adds():
    assert(add(2, 3) == 5)

@test
fn first_failure():
    assert(add(2, 2) == 5)

@test
@should_panic
fn panics_as_it_should():
    panic(\"boom\")

@test
@should_panic
fn does_not_panic():
    println(\"returned normally\")

@test
fn second_failure():
    println(\"printed by the second failure\")
    panic(\"second\")

fn main():
    println(\"main is not a test\")
");
    let all = ember(&["test"], &package);
    assert_ne!(all.exit, 0, "a failing test passed the run:\n{}{}", all.stdout, all.stderr);
    for line in [
        "running 5 tests",
        "test adds ... ok",
        "test first_failure ... FAILED",
        "test panics_as_it_should ... ok",
        "test does_not_panic ... FAILED",
        "test second_failure ... FAILED",
        "it was to panic, and returned",
        "printed by the second failure",
        "test result: FAILED. 2 passed; 3 failed; 0 quarantined; 0 filtered out",
    ] {
        assert!(all.stdout.contains(line), "missing `{line}`:\n{}{}", all.stdout, all.stderr);
    }
    // `[TST-30]` — the summary names every failure.
    let summary = all.stdout.rsplit("failures:\n").next().expect("a failure list");
    for name in ["first_failure", "does_not_panic", "second_failure"] {
        assert!(summary.contains(&format!("    {name}\n")), "`{name}` is not in the summary:\n{}", all.stdout);
    }
    assert!(!all.stdout.contains("main is not a test"), "the test binary ran `main`:\n{}", all.stdout);
    // The binary's C entry runs a test: a backtrace does not end in `main`.
    assert!(!all.stdout.contains(": main at"), "a test's backtrace names `main`:\n{}", all.stdout);

    let one = ember(&["test", "adds"], &package);
    assert_eq!(one.exit, 0, "{}{}", one.stdout, one.stderr);
    assert!(one.stdout.contains("test result: ok. 1 passed; 0 failed; 0 quarantined; 4 filtered out"), "{}", one.stdout);

    let json = ember(&["test", "panic", "--json"], &package);
    let report: serde_json::Value = serde_json::from_str(json.stdout.trim()).expect("`--json` prints one JSON value");
    assert_eq!(report["passed"], 1, "{report}");
    assert_eq!(report["failed"], 1, "{report}");
    assert_eq!(report["filtered_out"], 3, "{report}");

    let extra = ember(&["test", "adds", "first_failure"], &package);
    assert!(extra.exit != 0 && extra.stderr.contains("its one word is a filter"), "{}", extra.stderr);
    let lonely = temporary_directory("test-without-package");
    let none = ember(&["test"], &lonely);
    assert!(none.exit != 0 && none.stderr.contains("none has an"), "{}", none.stderr);
}

/// `[TST-30]` — a test that fails and then passes when run again fails
/// intermittently: it is quarantined by name, listed in the summary, and
/// does not fail the run. This one fails while a marker file exists, and
/// removes it as it fails.
#[test]
fn ember_test_quarantines_a_test_that_fails_intermittently() {
    let package = test_package("flaky", "lib", "\
unsafe extern \"C\":
    @ffi(param(path, borrowed, nul_terminated))
    safe fn remove(path: cstr) -> i32

@test
fn steady():
    assert(1 + 1 == 2)

@test
fn flaky():
    if remove(c\"marker\") == 0:
        panic(\"the marker was there\")
");
    std::fs::write(package.join("marker"), "fail once").expect("the marker is writable");
    let run = ember(&["test"], &package);
    assert_eq!(run.exit, 0, "a quarantined test failed the run:\n{}{}", run.stdout, run.stderr);
    for line in [
        "test steady ... ok",
        "test flaky ... FAILED intermittently: quarantined (failed 1 of 2 runs)",
        "quarantined: failed intermittently, not counted as failures [TST-30]:\n    flaky (failed 1 of 2 runs)",
        "the marker was there",
        "test result: ok. 1 passed; 0 failed; 1 quarantined; 0 filtered out",
    ] {
        assert!(run.stdout.contains(line), "missing `{line}`:\n{}{}", run.stdout, run.stderr);
    }
    // Without the marker it is an ordinary passing test.
    let again = ember(&["test"], &package);
    assert!(again.stdout.contains("test result: ok. 2 passed; 0 failed; 0 quarantined"), "{}", again.stdout);
}

/// `[WK-15]` — `ember test` in the debug profile reports the objects a test
/// leaked, and their cycles, when it ends; `--no-leak-check` turns that off,
/// and the development profile has it only when asked.
#[test]
fn ember_test_reports_leaks_unless_turned_off() {
    let package = test_package("leaky", "lib", "\
class Node:
    name: String
    next: Option[Node]

@test
fn builds_a_cycle():
    a = Node(\"a\", None)
    b = Node(\"b\", Some(a))
    a.next = Some(b)
");
    let default = ember(&["test"], &package);
    assert_eq!(default.exit, 0, "{}{}", default.stdout, default.stderr);
    assert!(
        default.stderr.contains("leak report of test builds_a_cycle [WK-15]:")
            && default.stderr.contains("runtime ownership cycle:"),
        "no leak report:\n{}{}", default.stdout, default.stderr
    );
    let off = ember(&["test", "--no-leak-check"], &package);
    assert!(off.exit == 0 && !off.stderr.contains("runtime ownership cycle"), "--no-leak-check reported:\n{}", off.stderr);
    let release = ember(&["test", "--profile", "development"], &package);
    assert!(release.exit == 0 && !release.stderr.contains("runtime ownership cycle"), "release reported unasked:\n{}", release.stderr);
    let asked = ember(&["test", "--profile", "development", "--leak-check"], &package);
    assert!(asked.exit == 0 && asked.stderr.contains("runtime ownership cycle"), "release --leak-check silent:\n{}", asked.stderr);
    let both = ember(&["test", "--leak-check", "--no-leak-check"], &package);
    assert!(both.exit != 0 && both.stderr.contains("contradict"), "{}", both.stderr);
}

/// `[DIA-6]`, `[DOC-2]` — `ember test --doc` builds each error page's
/// examples (the failing one must fail with the page's code, the fixed one
/// must compile, each in the manifest of the `toml` block above it) and runs
/// each user-guide sample. Every failure is reported.
#[test]
fn ember_test_doc_runs_the_documentation_examples() {
    let package = test_package("documented", "lib", "pub fn unused():\n    pass\n");
    let docs = package.join("docs");
    std::fs::create_dir_all(docs.join("errors")).expect("docs/errors is writable");
    std::fs::create_dir_all(docs.join("book").join("basics")).expect("docs/book is writable");
    std::fs::write(docs.join("errors").join("E3040.md"), "\
# E3040

```ember,fails
fn main():
    a = [1, 2, 3]
    b = a
    a.push(4)
```

## Why

Prose.

```ember,fixed
fn main():
    a = [1, 2, 3]
    b = a.clone()
    a.push(4)
    println(b.len())
```
").expect("a page is writable");
    // A page whose fix does not compile.
    std::fs::write(docs.join("errors").join("E2020.md"), "\
# E2020

```ember,fails
fn main():
    x: int = \"text\"
```

```ember,fixed
fn main():
    x: int = \"still text\"
```
").expect("a page is writable");
    // A manifest diagnostic: each example in the manifest above it.
    std::fs::write(docs.join("errors").join("E9010.md"), "\
# E9010

```toml
[lints]
not_a_lint = \"warn\"
```

```ember,fails
fn main():
    println(42)
```

```toml
[lints]
unused = \"warn\"
```

```ember,fixed
fn main():
    println(42)
```
").expect("a page is writable");
    // An error only a C compiler decides: valid Ember, which this C compiler
    // may build (then it is not reproduced here) or reject with the code.
    std::fs::write(docs.join("errors").join("E9041.md"), "\
# E9041

```ember,fails,toolchain
@fp(contract)
fn dot(a: f64, b: f64, c: f64, d: f64) -> f64:
    return a * b + c * d

fn main():
    println(dot(1.0, 2.0, 3.0, 4.0))
```

```ember,fixed
fn main():
    println(1.0)
```
").expect("a page is writable");
    std::fs::write(docs.join("book").join("basics").join("hello.md"), "\
# Hello

```ember
fn main():
    println(\"hello\")
```

```ember,fragment
fn uses(p: Point) -> int:
    return p.x
```

```ember,ignore
this is not Ember
```

```ember
fn main():
    panic(\"a sample that fails\")
```
").expect("a guide page is writable");
    let run = ember(&["test", "--doc"], &package);
    assert_ne!(run.exit, 0, "a failing example passed:\n{}{}", run.stdout, run.stderr);
    for line in [
        "running 11 examples",
        "example docs/errors/E2020.md:3 (fails) ... ok",
        "example docs/errors/E2020.md:8 (fixed) ... FAILED",
        "example docs/errors/E3040.md:3 (fails) ... ok",
        "example docs/errors/E3040.md:14 (fixed) ... ok",
        "example docs/errors/E9010.md:8 (fails) ... ok",
        "example docs/errors/E9010.md:18 (fixed) ... ok",
        "example docs/errors/E9041.md:3 (fails) ... ok",
        "example docs/errors/E9041.md:12 (fixed) ... ok",
        "example docs/book/basics/hello.md:3 ... ok",
        "example docs/book/basics/hello.md:8 ... ok",
        "example docs/book/basics/hello.md:17 ... FAILED",
        "the fixed program does not compile",
        "a sample that fails",
        "example result: FAILED. 9 passed; 2 failed; 0 quarantined; 0 filtered out",
    ] {
        assert!(run.stdout.contains(line), "missing `{line}`:\n{}{}", run.stdout, run.stderr);
    }
    let filtered = ember(&["test", "--doc", "E3040"], &package);
    assert_eq!(filtered.exit, 0, "{}{}", filtered.stdout, filtered.stderr);
    assert!(filtered.stdout.contains("2 passed; 0 failed; 0 quarantined; 9 filtered out"), "{}", filtered.stdout);
}

/// The header of the `ember bind` milestones: a pointer the header says nothing about, a scalar
/// function, a `const` and a mutable pointer, and a variadic function the importer skips.
const BIND_HEADER: &str = "\
#include <stddef.h>
static inline int bind_add(int a, int b) { return a + b; }
static inline int bind_sum(const int* items, size_t n) { int t = 0; for (size_t i = 0; i < n; i++) t += items[i]; return t; }
static inline void bind_zero(int* items, size_t n) { for (size_t i = 0; i < n; i++) items[i] = 0; }
int bind_log(const char* format, ...);
";

/// `[CLI-5]` — `ember bind <header> --init` writes a starter overlay: each fact the header cannot
/// state a `TODO(…)` word, each derived one a comment, each skipped declaration a comment with its
/// reason. It compiles unchanged (every call still `unsafe`), is never written over, and `--merge`
/// adds only what it lacks, keeping what was written by hand.
#[test]
fn bind_init_writes_a_starter_overlay_that_compiles_unchanged() {
    let dir = temporary_directory("bind-init");
    std::fs::write(dir.join("calc.h"), BIND_HEADER).unwrap();
    let init = ember(&["bind", "calc.h", "--init"], &dir);
    assert_eq!(init.exit, 0, "{}", init.stderr);
    let overlay = dir.join(format!("calc.overlay.{SOURCE_EXT}"));
    let written = std::fs::read_to_string(&overlay).expect("the starter overlay is written");
    for line in [
        "unsafe overlay c \"calc.h\":",
        "    fn bind_add(a, b) -> unsafe",
        "    # derived: `items` is a read-only borrow (`const`)",
        "    fn bind_sum(items: TODO(ownership) TODO(count) TODO(nullable), n)",
        "    # derived: `items` is a mutable borrow (no `const`)",
        "    # skipped: `bind_log` (calc.h:5): variadic",
    ] {
        assert!(written.contains(line), "the starter overlay lacks {line:?}:\n{written}");
    }
    // It compiles unchanged: every call is still the caller's assertion.
    std::fs::write(dir.join(ember_branding::source_file("main")), format!(
        "import c \"calc.h\" with (overlay = \"calc.overlay.{SOURCE_EXT}\")\n\nfn main():\n    \
         items: Array[i32] = [1, 2]\n    unsafe:  # SAFETY(ffi): two live items\n        \
         println(c.bind_add(1, 2), c.bind_sum(ref_to_ptr(ref items[0]), 2))\n")).unwrap();
    let checked = ember(&["check", &ember_branding::source_file("main")], &dir);
    assert_eq!(checked.exit, 0, "the starter overlay does not compile:\n{}", checked.stderr);
    // Never written over.
    std::fs::write(&overlay, written.replace("    fn bind_add(a, b) -> unsafe\n", "    fn bind_add(a, b)\n    # mine\n")).unwrap();
    let again = ember(&["bind", "calc.h", "--init"], &dir);
    assert_ne!(again.exit, 0, "an existing overlay was written over");
    // `--merge` adds the new function only.
    std::fs::write(dir.join("calc.h"), format!("{BIND_HEADER}static inline int bind_neg(int a) {{ return -a; }}\n")).unwrap();
    let merged = ember(&["bind", "calc.h", "--init", "--merge"], &dir);
    assert_eq!(merged.exit, 0, "{}", merged.stderr);
    let after = std::fs::read_to_string(&overlay).unwrap();
    assert!(after.contains("    fn bind_add(a, b)\n    # mine\n"), "a hand-written entry was lost:\n{after}");
    assert!(after.contains("    fn bind_neg(a) -> unsafe\n"), "the new function is missing:\n{after}");
    assert_eq!(after.matches("fn bind_sum(").count(), 1, "an entry was added twice:\n{after}");
    assert_eq!(after.matches("# skipped: `bind_log`").count(), 1, "a skip was noted twice:\n{after}");
}

/// `[CLI-6]` — `ember bind --report` lists every skipped declaration and every declaration still
/// `unsafe` with its unknown facts, and exits non-zero on a skip; `--baseline` limits that to skips
/// the baseline does not list. `[FFI-35a]` — it lists, as an assumption, every pointer parameter
/// the overlay gives no `retained` word: it does not retain the pointer. `[CLI-7]` — `ember bind
/// --check <overlay>` checks `[FFI-12]` and prints how many declarations are safe, still `unsafe`,
/// and skipped.
#[test]
fn bind_report_and_check_count_what_is_safe_unsafe_and_skipped() {
    let dir = temporary_directory("bind-report");
    std::fs::write(dir.join("calc.h"), BIND_HEADER).unwrap();
    std::fs::write(dir.join(format!("good.{SOURCE_EXT}")), "unsafe overlay c \"calc.h\":\n    fn bind_add(a, b)\n    fn bind_sum(items: borrowed count(n), n)\n    fn bind_zero(items: borrowed TODO(count), n)\n").unwrap();
    let report = ember(&["bind", "calc.h", "--report", "--overlay", &format!("good.{SOURCE_EXT}")], &dir);
    assert_ne!(report.exit, 0, "a skip did not fail the report:\n{}", report.stdout);
    for line in [
        "skipped: `bind_log` (calc.h:5): variadic",
        "unsafe: `bind_zero`: `items`: TODO(count)",
        "assumes: `bind_sum`: `items` does not retain the pointer",
        "1 skipped declaration(s)",
    ] {
        assert!(report.stdout.contains(line), "the report lacks {line:?}:\n{}", report.stdout);
    }
    assert!(!report.stdout.contains("unsafe: `bind_add`") && !report.stdout.contains("unsafe: `bind_sum`"), "{}", report.stdout);
    let written = ember(&["bind", "calc.h", "--report", "--write-baseline", "skips.txt"], &dir);
    assert!(written.stdout.contains("unsafe: `bind_add`: safe to call"), "{}", written.stdout);
    let baselined = ember(&["bind", "calc.h", "--report", "--baseline", "skips.txt"], &dir);
    assert_eq!(baselined.exit, 0, "a skip in the baseline failed the report:\n{}", baselined.stdout);

    let check = ember(&["bind", "calc.h", "--check", &format!("good.{SOURCE_EXT}")], &dir);
    assert_eq!(check.exit, 0, "{}", check.stderr);
    assert!(check.stdout.contains("2 safe, 1 still unsafe, 1 skipped"), "{}", check.stdout);
    std::fs::write(dir.join(format!("drifted.{SOURCE_EXT}")), "unsafe overlay c \"calc.h\":\n    fn bind_add(a)\n").unwrap();
    let drifted = ember(&["bind", "calc.h", "--check", &format!("drifted.{SOURCE_EXT}")], &dir);
    assert_ne!(drifted.exit, 0, "a drifted overlay passed the check");
    assert!(drifted.stderr.contains("E5010") && drifted.stderr.contains("`bind_add` takes 2 parameters"), "{}", drifted.stderr);
}

/// `[FFI-29c]` — a call to a wrapped `static inline` function costs one extra call: `ember inspect
/// --cost` reports it as `wrapped (static inline)`, beside an ordinary foreign call's one direct
/// call; a variadic `static inline` function cannot be wrapped and is skipped with `W5002`.
#[test]
fn inspect_reports_a_wrapped_static_inline_call() {
    let root = workspace_root();
    let dir = temporary_directory("inspect-wrapped");
    std::fs::write(dir.join("wrapped.h"), "\
static inline int wrapped_twice(int x) { return 2 * x; }
static inline int wrapped_sum(int n, ...) { return n; }
int wrapped_plain(int x);
").unwrap();
    std::fs::write(dir.join("wrapped.c"), "#include \"wrapped.h\"\nint wrapped_plain(int x) { return x + 1; }\n").unwrap();
    let source = dir.join(format!("calls.{SOURCE_EXT}"));
    std::fs::write(&source, "import c \"./wrapped.c\"\n\nfn main():\n    unsafe:  # SAFETY(ffi): scalar calls\n        \
        println(c.wrapped_twice(2), c.wrapped_plain(2))\n").unwrap();
    let out_dir = dir.join("out");
    let build = ember(&["build", &source.to_string_lossy(), "--out-dir", &out_dir.to_string_lossy()], &root);
    assert_eq!(build.exit, 0, "build failed:\n{}", build.stderr);
    assert!(build.stderr.contains("W5002") && build.stderr.contains("`wrapped_sum` is not imported") && build.stderr.contains("variadic"),
        "the variadic static inline function was not reported:\n{}", build.stderr);
    let table = out_dir.join("debug").join("inspect").join("calls.safety.json").to_string_lossy().into_owned();
    let report = ember(&["inspect", &table, "--cost"], &root);
    assert_eq!(report.exit, 0, "{}", report.stderr);
    assert!(report.stdout.contains("wrapped_twice`, wrapped (static inline)"), "{}", report.stdout);
    assert!(report.stdout.contains("wrapped_plain`, one direct call"), "{}", report.stdout);
}

/// `[FFI-7]` — a header is parsed again only when its content, its overlay, or a setting that can
/// change a binding (its defines) changes; otherwise the import is read from its `.embind` record.
/// `[FFI-14]` — the record is versioned CBOR holding the header's files and their hashes, the Clang
/// configuration, the overlay's hash, and the declarations with their layouts; `ember bind
/// --emit-embind` writes one, and `ember bind --explain <fn>` prints a function's contract and why.
#[test]
fn import_c_parses_a_header_again_only_when_it_changes() {
    let root = workspace_root();
    let dir = temporary_directory("embind");
    let cache = temporary_directory("embind-cache");
    let cache_arg = cache.to_string_lossy().into_owned();
    std::fs::write(dir.join("cached.h"), "typedef struct Pair { int a; int b; } Pair;\nstatic inline int cached_sum(const int* items, int n) { return n > 0 ? items[0] : 0; }\nstatic inline int cached_one(int x) { return x; }\n").unwrap();
    std::fs::write(dir.join(format!("overlay.{SOURCE_EXT}")), "unsafe overlay c \"cached.h\":\n    fn cached_sum(items: TODO(ownership) TODO(count) TODO(nullable), n)\n").unwrap();
    let source = dir.join(format!("main.{SOURCE_EXT}"));
    let program = |defines: &str| format!("import c \"cached.h\" with (overlay = \"overlay.{SOURCE_EXT}\"{defines})\n\nfn main():\n    p = c.Pair(a=1, b=2)\n    println(p.a + p.b)\n");
    std::fs::write(&source, program("")).unwrap();
    let source_arg = source.to_string_lossy().into_owned();
    let check = || ember_with_env(&["check", &source_arg, "--timings"], &root, &ember_branding::cache_dir_var(), &cache_arg);
    let parsed = |run: &Run| run.stdout.contains("import c, parsed") || run.stderr.contains("import c, parsed");
    let from_record = format!("import c, from its .{}", ember_branding::BINDING_EXT);
    let reused = |run: &Run| run.stdout.contains(&from_record) || run.stderr.contains(&from_record);
    let first = check();
    assert_eq!(first.exit, 0, "{}", first.stderr);
    assert!(parsed(&first), "the first build did not parse the header:\n{}{}", first.stdout, first.stderr);
    let second = check();
    assert!(reused(&second) && !parsed(&second), "nothing changed, and the header was parsed again:\n{}{}", second.stdout, second.stderr);
    // The header's content.
    std::fs::write(dir.join("cached.h"), "typedef struct Pair { int a; int b; } Pair;\nstatic inline int cached_sum(const int* items, int n) { return n; }\nstatic inline int cached_one(int x) { return x; }\n").unwrap();
    assert!(parsed(&check()), "a changed header was not parsed again");
    assert!(reused(&check()));
    // Its overlay.
    std::fs::write(dir.join(format!("overlay.{SOURCE_EXT}")), "unsafe overlay c \"cached.h\":\n    fn cached_sum(items: TODO(ownership) TODO(count) TODO(nullable), n)\n    fn cached_one(x)\n").unwrap();
    assert!(parsed(&check()), "a changed overlay did not parse the header again");
    // A define.
    std::fs::write(&source, program(", defines = [\"CACHED_EXTRA=1\"]")).unwrap();
    assert!(parsed(&check()), "a new define did not parse the header again");
    // The record: versioned CBOR, a map whose first key is `version`.
    let records: Vec<PathBuf> = std::fs::read_dir(cache.join(ember_branding::BINDING_EXT)).expect("the records' folder").filter_map(|entry| entry.ok().map(|entry| entry.path())).collect();
    assert!(!records.is_empty());
    let bytes = std::fs::read(&records[0]).unwrap();
    assert_eq!(bytes[0] >> 5, 5, "not a CBOR map");
    assert_eq!(&bytes[1..9], b"\x67version", "the record does not open with its version");
    for wanted in ["files", "clang", "overlay", "contracts", "layouts", "cached.h"] {
        assert!(bytes.windows(wanted.len()).any(|window| window == wanted.as_bytes()), "the record lacks {wanted}");
    }
    let emitted = ember(&["bind", "cached.h", "--emit-embind"], &dir);
    assert_eq!(emitted.exit, 0, "{}", emitted.stderr);
    assert!(dir.join(format!("cached.{}", ember_branding::BINDING_EXT)).is_file());
    let explained = ember(&["bind", "cached.h", "--explain", "cached_sum", "--overlay", &format!("overlay.{SOURCE_EXT}")], &dir);
    assert_eq!(explained.exit, 0, "{}", explained.stderr);
    for line in [
        "`items`: `const int *` — a read-only borrow (derived: `const`); stated by the overlay: TODO(ownership) TODO(count) TODO(nullable)",
        "`n`: `int` — a value: nothing to state",
        "called through the import's shim",
    ] {
        assert!(explained.stdout.contains(line), "--explain lacks {line:?}:\n{}", explained.stdout);
    }
}

/// `[FFI-30b]` — the package's interface hash (`[BLD-2]`) includes the layouts of what `pub import c`
/// re-exports: the header's module is a dependency of the re-exporting module, its interface holds
/// each imported struct's layout, and a header change that changes a layout changes the hash its
/// dependants record, so they are rebuilt.
#[test]
fn a_reexported_header_layout_is_part_of_the_interface_hash() {
    let dir = temporary_directory("reexport-interface");
    let out_dir = dir.join("out");
    std::fs::write(dir.join("shape.h"), "typedef struct Shape { int w; int h; } Shape;\n").unwrap();
    std::fs::write(dir.join(format!("lib.{SOURCE_EXT}")), "pub import c \"shape.h\" as shape\n\npub fn area(s: shape.Shape) -> i32:\n    return s.w * s.h\n").unwrap();
    std::fs::write(dir.join(format!("main.{SOURCE_EXT}")), "import lib\n\nfn main():\n    println(lib.area(lib.shape.Shape(w=2, h=3)))\n").unwrap();
    let main = format!("main.{SOURCE_EXT}");
    let check = || {
        let run = ember(&["check", &main, "--out-dir", &out_dir.to_string_lossy()], &dir);
        assert_eq!(run.exit, 0, "{}", run.stderr);
    };
    check();
    let before = cached_interface(&out_dir, "lib");
    let header_hash = before.dependencies.get("c.h_shape").cloned().expect("`lib` depends on the header's module");
    let layouts = cached_interface(&out_dir, "c.h_shape").layouts;
    assert!(layouts.iter().any(|(name, layout)| name.contains("Shape") && layout.contains("pub h: i32")), "{layouts:?}");
    std::fs::write(dir.join("shape.h"), "typedef struct Shape { int w; int h; int d; } Shape;\n").unwrap();
    std::fs::write(dir.join(format!("main.{SOURCE_EXT}")), "import lib\n\nfn main():\n    println(lib.area(lib.shape.Shape(w=2, h=3, d=4)))\n").unwrap();
    check();
    let after = cached_interface(&out_dir, "lib");
    assert_ne!(after.dependencies.get("c.h_shape"), Some(&header_hash), "a changed layout left the dependency's hash as it was");
}

/// `[TCB-5]` — an instrumented fact of a declaration `import c` wrote is valid only for the header
/// and overlay it was measured against: the evidence record stores their paths and hashes, and a
/// changed header makes it stale (`[TCB-6]`), so the fact goes back to asserted.
#[test]
fn evidence_for_an_imported_declaration_rests_on_its_header_and_overlay() {
    let package = test_package("imported_measured", "bin", "\
import c \"measure.h\" with (overlay = \"overlays/measure.em\")

@test
fn twice():
    assert(c.measure_twice(4) == 8)

fn main():
    println(c.measure_twice(1))
");
    std::fs::write(package.join("src").join("measure.h"), "static inline int measure_twice(int x) { return 2 * x; }\n").unwrap();
    std::fs::create_dir_all(package.join("overlays")).unwrap();
    std::fs::write(package.join("overlays").join(format!("measure.{SOURCE_EXT}")),
        "unsafe overlay c \"measure.h\":\n    @ffi(effects=[])\n    fn measure_twice(x)\n").unwrap();
    let out_arg = package.join("target").to_string_lossy().into_owned();
    let run = ember(&["test", "--instrument-ffi", "--out-dir", &out_arg], &package);
    assert_eq!(run.exit, 0, "{}{}", run.stdout, run.stderr);
    let record_path = package.join(format!(".{}/ffi-evidence/main.json", ember_branding::SYMBOL_PREFIX));
    let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&record_path).expect("the evidence record is written"))
        .expect("the evidence record is JSON");
    let function = record["functions"].as_array().expect("a function list").iter()
        .find(|function| function["name"].as_str().is_some_and(|name| name.ends_with("measure_twice")))
        .expect("`measure_twice` is recorded");
    let roles: Vec<&str> = function["imported_from"].as_array().expect("its identities").iter()
        .filter(|identity| identity["hash"].is_string())
        .filter_map(|identity| identity["role"].as_str())
        .collect();
    assert_eq!(roles, ["header", "overlay"], "{function}");
    let entry = package.join("src").join(ember_branding::source_file("main")).to_string_lossy().into_owned();
    let fresh = ember(&["tcb", &entry], &package);
    assert!(fresh.stdout.contains("no Alloc: instrumented"), "{}{}", fresh.stdout, fresh.stderr);
    std::fs::write(package.join("src").join("measure.h"), "static inline int measure_twice(int x) { return x + x; }\n").unwrap();
    let stale = ember(&["tcb", &entry], &package);
    assert!(stale.stdout.contains("evidence stale") && stale.stdout.contains("its header"), "{}{}", stale.stdout, stale.stderr);
}

/// `[TCB-1]` — a grade written on an overlay entry is a claim. `@grade(instrumented)` with no
/// evidence record is `W5050`, and `E5050` under `ember tcb --require` or `ember build
/// --require-evidence`; once `ember test --instrument-ffi` has recorded the function's effect facts
/// the claim is met; a changed header makes the record stale (`[TCB-6]`) and the claim unmet again.
#[test]
fn a_grade_on_an_overlay_entry_is_met_only_by_evidence_that_holds() {
    let package = test_package("graded", "bin", "\
import c \"grade.h\" with (overlay = \"overlays/grade.em\")

@test
fn doubles():
    assert(c.grade_twice(2) == 4)

fn main():
    println(c.grade_twice(1))
");
    std::fs::write(package.join("src").join("grade.h"), "static inline int grade_twice(int x) { return 2 * x; }\n").unwrap();
    std::fs::create_dir_all(package.join("overlays")).unwrap();
    std::fs::write(package.join("overlays").join(format!("grade.{SOURCE_EXT}")),
        "unsafe overlay c \"grade.h\":\n    @grade(instrumented)\n    @ffi(effects=[])\n    fn grade_twice(x)\n").unwrap();
    let entry = package.join("src").join(ember_branding::source_file("main")).to_string_lossy().into_owned();
    let out_arg = package.join("target").to_string_lossy().into_owned();
    let claimed = "`instrumented` is claimed with no evidence";

    let unbacked = ember(&["check", &entry], &package);
    assert!(unbacked.exit == 0 && unbacked.stderr.contains("warning[W5050]") && unbacked.stderr.contains(claimed), "{}", unbacked.stderr);
    let required = ember(&["tcb", "--require", &entry], &package);
    assert!(required.exit != 0 && required.stderr.contains("error[E5050]"), "{}{}", required.stdout, required.stderr);
    let build = ember(&["build", &entry, "--require-evidence", "--out-dir", &out_arg], &package);
    assert!(build.exit != 0 && build.stderr.contains("error[E5050]"), "{}", build.stderr);

    let run = ember(&["test", "--instrument-ffi", "--out-dir", &out_arg], &package);
    assert_eq!(run.exit, 0, "{}{}", run.stdout, run.stderr);
    let backed = ember(&["check", &entry], &package);
    assert!(backed.exit == 0 && !backed.stderr.contains("W5050"), "{}", backed.stderr);
    let met = ember(&["tcb", "--require", &entry], &package);
    assert_eq!(met.exit, 0, "{}{}", met.stdout, met.stderr);

    std::fs::write(package.join("src").join("grade.h"), "static inline int grade_twice(int x) { return x + x; }\n").unwrap();
    let stale = ember(&["check", &entry], &package);
    assert!(stale.stderr.contains("warning[W5050]"), "a stale record still met the claim:\n{}", stale.stderr);
}

/// `[TST-7]` — every ```` ```ember ```` block of the development target is checked: a plain block
/// with `ember check` (one holding `#$ test: compile-fail` as the test it is, its `#$ error[…]`
/// diagnostics on their lines), an `ember,fragment` with `--syntax-only`, an `ember,overlay` as the
/// overlay it is, and an `ember,ignore` only with its reason, a signature sketch or a deliberate
/// error example, stated on its first line. `[TST-29]` — the baseline of blocks known to fail is
/// printed with its size, and a block in it that passes fails this test until it is taken out.
#[test]
fn every_example_of_the_specification_checks() {
    // A block known to fail, by the heading it is under, and why.
    const BASELINE: &[(&str, &str)] = &[
        ("Part XVI", "`std.ffi` has no `c_int` or `c_double` yet (D-742)"),
    ];
    let root = workspace_root();
    let pinned = std::fs::read_to_string(root.join("docs/spec-source/development-target.json")).expect("the pin");
    let path = pinned.split('"').skip_while(|part| *part != "path").nth(2).expect("the pin names the target");
    let text = std::fs::read_to_string(root.join(path)).expect("the development target");
    let lines: Vec<&str> = text.lines().collect();
    let directory = temporary_directory("spec-examples");
    let (mut heading, mut at, mut checked, mut failures, mut baselined) = (String::new(), 0, 0, Vec::new(), Vec::new());
    while at < lines.len() {
        if lines[at].starts_with('#') {
            heading = lines[at].trim_start_matches('#').trim().to_string();
        }
        let Some(kind) = lines[at].strip_prefix("```ember") else {
            at += 1;
            continue;
        };
        let start = at + 1;
        let end = start + lines[start..].iter().position(|line| line.starts_with("```")).expect("every block closes");
        let body = lines[start..end].join("\n") + "\n";
        at = end + 1;
        let file = directory.join(format!("block{start}.{SOURCE_EXT}"));
        std::fs::write(&file, &body).expect("the block is written");
        let file = file.to_string_lossy().into_owned();
        let where_ = format!("line {start} ({heading})");
        let failed = match kind.trim().trim_start_matches(',') {
            "ignore" => {
                let reason = lines[start].to_lowercase();
                if !(reason.starts_with("##") && (reason.contains("signature sketch") || reason.contains("deliberate error"))) {
                    failures.push(format!("{where_}: `ember,ignore` without a stated reason on its first line"));
                }
                continue;
            }
            "fragment" => {
                let run = ember(&["check", "--syntax-only", &file], &directory);
                (run.exit != 0).then_some(run.stderr)
            }
            "overlay" | "" => {
                let run = ember(&["check", &file], &directory);
                if body.contains("#$ test: compile-fail") {
                    let expected: Vec<(usize, &str)> = body
                        .lines()
                        .enumerate()
                        .filter_map(|(index, line)| line.split_once("#$ error[").map(|(_, rest)| (index + 1, rest.trim())))
                        .collect();
                    let missing: Vec<String> = expected
                        .iter()
                        .filter(|(line, message)| {
                            !run.stderr.contains(&format!("error[{message}")) || !run.stderr.contains(&format!(":{line}:"))
                        })
                        .map(|(line, message)| format!("line {line}: error[{message}"))
                        .collect();
                    (run.exit == 0 || !missing.is_empty()).then(|| format!("expected {missing:?}\n{}", run.stderr))
                } else {
                    (run.exit != 0).then_some(run.stderr)
                }
            }
            other => Some(format!("an unknown block kind `ember,{other}`")),
        };
        checked += 1;
        let known = BASELINE.iter().find(|(part, _)| heading.starts_with(part));
        match (failed, known) {
            (Some(_), Some((_, why))) => baselined.push(format!("{where_}: {why}")),
            (None, Some(_)) => failures.push(format!("{where_} passes now: take it out of the baseline")),
            (Some(error), None) => failures.push(format!("{where_}:\n{error}")),
            (None, None) => {}
        }
    }
    println!("[TST-7] {checked} blocks checked; baseline {} of them ([TST-29]):", baselined.len());
    for entry in &baselined {
        println!("  {entry}");
    }
    assert!(checked > 30, "the development target's blocks were not found ({checked})");
    assert!(failures.is_empty(), "{} specification example(s) fail:\n{}", failures.len(), failures.join("\n\n"));
}

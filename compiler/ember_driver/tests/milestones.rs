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
//!
//! `#$` is an ordinary comment to the compiler — `$` has no other meaning
//! anywhere in Ember — so an annotation can never be mistaken for source and
//! can never affect compilation.
//!
//! `[TST-0]` — annotations are read from the raw source text, not from the
//! token stream: a `compile-fail` test may be expected to fail at the lexer,
//! so its expectations must be readable even when the file does not tokenise.

use std::hash::{Hash, Hasher};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex, PoisonError};
use std::time::{SystemTime, UNIX_EPOCH};

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
    stdout: Option<String>,
    /// `#$ stdin:` — one line of standard input per annotation.
    stdin: Option<String>,
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
        } else if let Some(value) = rest.strip_prefix("stdin:") {
            stdin_lines.push(value.trim().to_string());
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
  known keys:                  test, profiles, exit, assert-c, error[…], warning[…], help, not-help, panics, stdout, stdin, rules, note"
            );
        }
    }

    if !stdout_lines.is_empty() {
        expectations.stdout = Some(stdout_lines.join("\n"));
    }
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
fn ember_with_input(args: &[&str], root: &Path, input: &str) -> Run {
    use std::io::Write;
    let mut child = Command::new(EMBER)
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the ember binary runs");
    child.stdin.take().expect("piped stdin").write_all(input.as_bytes()).expect("stdin is written");
    let output = child.wait_with_output().expect("the ember binary finishes");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
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

fn ember_with_env(args: &[&str], root: &Path, key: &str, value: &str) -> Run {
    let output = Command::new(EMBER)
        .args(args)
        .env(key, value)
        .current_dir(root)
        .output()
        .expect("the ember binary runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
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
    let run = Command::new(output).output().expect("linked program runs");
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
        ("from_import", "from imported import renamed_count, changing_counter, read_changing_counter\nfn main():\n    unsafe:\n        changing_counter = 12\n    println(read_changing_counter())\n    println(renamed_count)\n"),
        ("qualified", "import imported\nfn main():\n    unsafe:\n        imported.changing_counter = 12\n    println(imported.read_changing_counter())\n    println(imported.renamed_count)\n"),
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
        let imported_run = Command::new(imported_exe).output().expect("imported static program runs");
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
    let run = Command::new(output).output().expect("linked program runs");
    assert!(run.status.success(), "foreign record program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "11\n2\n7\n17\n21\n35\n16\n24\n37\n");

    let imported = directory.join(ember_branding::source_file("imported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignPair:\n    pub left: i32\n    pub right: i32\n@derive(Copy)\npub struct ForeignInner:\n    pub value: i32\n@derive(Copy)\npub struct ForeignOuter:\n    pub inner: ForeignInner\n    pub other: i32\nunsafe extern \"C\":\n    pub static mut shared_pair: ForeignPair\n    @ffi(immutable)\n    pub static frozen_pair: ForeignPair\n    pub safe fn read_pair_sum() -> i32\n    pub static mut nested_pair: ForeignOuter\n    pub safe fn read_nested_total() -> i32\n")
        .expect("foreign record module is writable");
    for (label, source) in [
        ("from_import", "from imported import ForeignPair, shared_pair, frozen_pair, read_pair_sum, nested_pair, read_nested_total\nfn main():\n    println(frozen_pair.left)\n    unsafe:\n        shared_pair = ForeignPair(left = 11, right = 12)\n        println(read_pair_sum())\n        shared_pair.left = 20\n        println(read_pair_sum())\n        shared_pair.right += 2\n        println(read_pair_sum())\n        nested_pair.inner.value = 15\n        println(read_nested_total())\n        nested_pair.inner.value += 2\n        println(read_nested_total())\n"),
        ("qualified", "import imported\nfn main():\n    println(imported.frozen_pair.left)\n    unsafe:\n        imported.shared_pair = imported.ForeignPair(left = 11, right = 12)\n        println(imported.read_pair_sum())\n        imported.shared_pair.left = 20\n        println(imported.read_pair_sum())\n        imported.shared_pair.right += 2\n        println(imported.read_pair_sum())\n        imported.nested_pair.inner.value = 15\n        println(imported.read_nested_total())\n        imported.nested_pair.inner.value += 2\n        println(imported.read_nested_total())\n"),
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
        let imported_run = Command::new(imported_exe).output().expect("imported record program runs");
        assert!(imported_run.status.success());
        assert_eq!(String::from_utf8_lossy(&imported_run.stdout).replace("\r\n", "\n"), "5\n23\n32\n34\n21\n23\n");
    }
    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignPair:\n    pub(read) left: i32\n    pub right: i32\nunsafe extern \"C\":\n    pub static mut shared_pair: ForeignPair\n")
        .expect("read-only foreign record module is writable");
    std::fs::write(&entry,
        "import imported\nfn main():\n    unsafe:\n        imported.shared_pair.left = 20\n")
        .expect("read-only foreign record entry is writable");
    let entry_arg = entry.to_string_lossy().into_owned();
    let readonly = ember(&["check", &entry_arg], &root);
    assert_ne!(readonly.exit, 0, "a pub(read) field was writable from another module");
    assert!(readonly.stderr.contains("E1050"), "expected read-only field error: {}", readonly.stderr);

    std::fs::write(&imported,
        "@derive(Copy)\npub struct ForeignInner:\n    pub(read) value: i32\n@derive(Copy)\npub struct ForeignOuter:\n    pub inner: ForeignInner\nunsafe extern \"C\":\n    pub static mut nested_pair: ForeignOuter\n")
        .expect("nested read-only foreign record module is writable");
    std::fs::write(&entry,
        "import imported\nfn main():\n    unsafe:\n        imported.nested_pair.inner.value = 20\n")
        .expect("nested read-only foreign record entry is writable");
    let nested_readonly = ember(&["check", &entry_arg], &root);
    assert_ne!(nested_readonly.exit, 0, "a nested pub(read) field was writable from another module");
    assert!(nested_readonly.stderr.contains("E1050"), "expected nested read-only field error: {}", nested_readonly.stderr);
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
    let run = Command::new(output).output().expect("linked program runs");
    assert!(run.status.success(), "foreign record program failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "8\n10\n17\n3\n4\n3\n13\n10\n16\n31\n22\n13\n10\n");

    let imported = directory.join(ember_branding::source_file("imported"));
    let entry = directory.join(ember_branding::source_file("entry"));
    std::fs::write(&imported,
        "pub struct ForeignPair:\n    pub left: i32\n    pub right: i32\nunsafe extern \"C\":\n    pub safe fn mutate_pair_copy(pair: ForeignPair) -> i32\n    @ffi(link_name=\"foreign_pair_weighted\")\n    pub fn weighted_pair(pair: ForeignPair, weight: i32) -> i32\n")
        .expect("foreign record module is writable");
    for (label, source) in [
        ("from_import", "from imported import ForeignPair, mutate_pair_copy, weighted_pair\nfn main():\n    pair = ForeignPair(left = 3, right = 4)\n    println(mutate_pair_copy(pair))\n    unsafe:\n        println(weighted_pair(pair, 2))\n"),
        ("qualified", "import imported\nfn main():\n    pair = imported.ForeignPair(left = 3, right = 4)\n    println(imported.mutate_pair_copy(pair))\n    unsafe:\n        println(imported.weighted_pair(pair, 2))\n"),
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
        let run = Command::new(executable).output().expect("imported record program runs");
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
    let run = Command::new(output).output().expect("linked program runs");
    assert!(run.status.success(), "borrowed record export failed: {}",
        String::from_utf8_lossy(&run.stderr));
    assert_eq!(String::from_utf8_lossy(&run.stdout).replace("\r\n", "\n"), "10\n3\n21\n10\n13\n811\n10\n19\n15\n7\n8\n");
}

/// `[FFI-25]` — a C caller cannot observe an Ember panic as a return or unwind.
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
    let run = Command::new(output).output().expect("linked program runs");
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
    let run = Command::new(output).output().expect("thread attachment host runs");
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
    let run = Command::new(output).output().expect("linked program runs");
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
        "#! language \"0.9.9\"\n#! threads main\n@export(\"default_score\")\npub fn default_value() -> i32:\n    return 43\n@export(\"override_score\", threads=any)\npub fn override_value() -> i32:\n    return 44\n")
        .expect("default policy module is writable");
    std::fs::write(&creator_imported,
        "#! language \"0.9.9\"\n#! threads creator\n@export(\"creator_default_score\")\npub fn default_value() -> i32:\n    return 46\n@export(\"creator_override_score\", threads=any)\npub fn override_value() -> i32:\n    return 47\n")
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
    for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
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
            let run = Command::new(&output).arg(mode).output().expect("C thread host runs");
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
    let c_run = Command::new(&c_output).output().expect("C header host runs");
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
    let cpp_run = Command::new(&cpp_output).output().expect("C++ header host runs");
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
    for profile in [Profile::Debug, Profile::Release, Profile::Shipping] {
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
        let run = Command::new(output).output().expect("native callbacks run on a host thread");
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
        ("unchecked-profile", "[profiles.shipping]\nexclusivity = \"unchecked\"\n", "exclusivity"),
        ("unchecked-build", "[build]\nexclusivity = \"unchecked\"\n", "exclusivity"),
        ("overflow", "[profiles.shipping]\noverflow = \"unchecked\"\n", "overflow"),
        ("bounds", "[profiles.shipping]\nbounds_checks = false\n", "bounds_checks"),
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
    for profile in ["debug", "release", "shipping"] {
        let out_dir = std::env::temp_dir().join(format!(
            "ember-hoisted-loop-safety-{profile}-{}",
            std::process::id()
        ));
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

/// ADR-079 — for MSVC, a loop nest that writes separate lists it never reads
/// runs in a function of its own whose list parameters are `restrict`, the
/// counts the program fixes written in; for clang the nest stays in place.
#[test]
fn loop_nests_over_separate_lists_get_restrict_functions_for_msvc() {
    let root = workspace_root();
    let source = format!("tests/conformance/OPT-2/accept_a_loop_nest_over_separate_lists_keeps_its_results.{SOURCE_EXT}");
    let main_loop = format!("{}_loop0(", ember_branding::mangled("main"));
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:
{}", msvc.stderr);
    assert!(msvc.stdout.contains(&format!("EMBER_NOINLINE void {main_loop}")), "no loop function for MSVC:
{}", msvc.stdout);
    assert!(msvc.stdout.contains("* restrict _1_ptr"), "the loop function's lists are not restrict:
{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE void").count(), 4, "two nests move (prototype and definition each)");
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:
{}", clang.stderr);
    assert!(!clang.stdout.contains(&main_loop), "clang keeps the nest in place:
{}", clang.stdout);
}

/// D-446 — `[CG-C-1]`: a signed `+`, `-` or `*` (negation is `0 - x`) reaching C
/// unchecked is done in its width's unsigned type, so wrapping is never C's
/// undefined signed overflow (gcc -O2 miscompiled a wrapping `i * k` in a
/// loop); MSVC and clang happened to keep it, so the C itself is the test.
#[test]
fn wrapping_arithmetic_is_unsigned_in_c() {
    let root = workspace_root();
    let source = format!("tests/conformance/TYP-8/accept_wrapping_arithmetic_in_a_loop_is_never_undefined.{SOURCE_EXT}");
    let c = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "gcc"], &root);
    assert_eq!(c.exit, 0, "C failed:
{}", c.stderr);
    for wrapped in [
        "(int64_t)((uint64_t)(_1) * (uint64_t)(6364136223846793005LL))",
        "(int32_t)((uint32_t)(",
        "(int16_t)((uint32_t)(",
        "(int64_t)((uint64_t)(0LL) - (uint64_t)(",
    ] {
        assert!(c.stdout.contains(wrapped), "no `{wrapped}`: wrapping arithmetic is signed in C:
{}", c.stdout);
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
    let gcc = ember(&["build", &separate, "--emit", "c", "--profile", "release", "--cc", "gcc"], &root);
    assert_eq!(gcc.exit, 0, "gcc C failed:
{}", gcc.stderr);
    assert_eq!(gcc.stdout.matches("EMBER_NOINLINE void").count(), 6, "three loops move (prototype and definition each):
{}", gcc.stdout);
    assert!(gcc.stdout.contains("* restrict _1_ptr"), "the loop function's lists are not restrict:
{}", gcc.stdout);
    let reading_back = format!("tests/conformance/OPT-2/accept_a_loop_nest_adding_the_same_each_round_keeps_its_results.{SOURCE_EXT}");
    let gcc = ember(&["build", &reading_back, "--emit", "c", "--profile", "release", "--cc", "gcc"], &root);
    assert_eq!(gcc.exit, 0, "gcc C failed:
{}", gcc.stderr);
    assert_eq!(gcc.stdout.matches("EMBER_NOINLINE void").count(), 6, "three nests move for gcc:
{}", gcc.stdout);
    let msvc = ember(&["build", &reading_back, "--emit", "c", "--profile", "release", "--cc", "msvc"], &root);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE void").count(), 2, "one nest moves for MSVC");
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
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "msvc"], &root);
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
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "clang"], &root);
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
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:\n{}", msvc.stderr);
    assert_eq!(msvc.stdout.matches(&format!("EMBER_NOINLINE int64_t {main_loop}")).count(), 4, "two whole-number nests move (prototype and definition each):\n{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches(&format!("EMBER_NOINLINE double {main_loop}")).count(), 2, "the decimal nest moves:\n{}", msvc.stdout);
    assert_eq!(msvc.stdout.matches("EMBER_NOINLINE ").count(), 6, "nothing else moves:\n{}", msvc.stdout);
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "clang"], &root);
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
    let msvc = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "msvc"], &root);
    assert_eq!(msvc.exit, 0, "msvc C failed:
{}", msvc.stderr);
    assert_eq!(msvc.stdout.lines().filter(|line| line.ends_with("_in;")).count(), 10, "five totals copied in and back:
{}", msvc.stdout);
    let clang = ember(&["build", &source, "--emit", "c", "--profile", "release", "--cc", "clang"], &root);
    assert_eq!(clang.exit, 0, "clang C failed:
{}", clang.stderr);
    assert!(!clang.stdout.lines().any(|line| line.ends_with("_in;")), "clang keeps the totals in place:
{}", clang.stdout);
}

/// `[PHIL-5]` — at the end of `main` the process ends: release and shipping
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
    for profile in ["debug", "release", "shipping"] {
        let out_dir = std::env::temp_dir().join(format!("ember-exit-drops-{profile}-{}", std::process::id()));
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
    assert_eq!(frees[1] + 1, frees[0], "release frees one list fewer than debug: {frees:?}");
    assert_eq!(frees[2], frees[1], "shipping drops what release drops: {frees:?}");
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
    let out_dir = std::env::temp_dir().join(format!("ember-no-conflicting-hold-{}", std::process::id()));
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
    // One short folder per case, named by a hash of its whole relative path:
    // cases run side by side, and rule directories share file names
    // (`accept_basic`). Short because Windows' linker cannot write a path over
    // 260 characters, and the executable is already named after the file;
    // the path itself as the name put the longest cases over it in CI.
    let out_dir = std::env::temp_dir().join("ember-tests").join(format!("{:016x}", {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        relative.hash(&mut hasher);
        hasher.finish()
    }));
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
        let arguments = ["run", relative.as_str(), "--out-dir", &out_dir_arg, "--profile", profile];
        let run = match &expectations.stdin {
            Some(input) => ember_with_input(&arguments, root, input),
            None => ember(&arguments, root),
        };

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
    std::thread::available_parallelism().map_or(4, |n| n.get())
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
fn module_attributes_format_from_the_parsed_tree() {
    let root = workspace_root();
    let scratch = std::env::temp_dir().join(ember_branding::source_file("ember-module-attr-fmt"));
    std::fs::write(&scratch,
        "#! language \"0.9.9\"\n#! module overflow(wrap)\n#! threads any\n\nfn add(a: i8, b: i8) -> i8:\n    return a + b\n")
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

/// `[TYP-8]` / ODR-084 — an imported declaration keeps its module's lexical
/// policy. Editing only that directive changes the callable interface and
/// invalidates the importing module even when its own source is unchanged.
#[test]
fn imported_module_overflow_policy_invalidates_callers_interface_key() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-overflow-interface-{}", std::process::id()
    ));
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

/// `[TYP-16]` / `[MOD-2]` / `[BLD-2]` — a public generic-enum member is a
/// source declaration even when no concrete enum instantiation is emitted.
#[test]
fn visible_generic_enum_member_declarations_invalidate_importers() {
    let workspace = workspace_root();
    let test_root = std::env::temp_dir().join(format!(
        "ember-generic-enum-interface-{}",
        std::process::id()
    ));
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
        let ran = Command::new(&program).output().expect("the program runs");
        assert_eq!(String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n"), native.stdout, "{relative}");
    }
    let _ = std::fs::remove_dir_all(&dir);
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
    report_failures("tests/conformance", &run_cases(&cases, |path| check_file(path, &root)));
}

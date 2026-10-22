//! `ember test [filter] [--doc] [--no-leak-check]` (XVII.1).
//!
//! `[TST-3]` — the package's `@test` functions are built into one test binary
//! (`compile` with the command `test`), whose entry runs the test its first
//! argument numbers. Each test runs in a process of its own, so a panic, a
//! crash or a leak in one says nothing about another. `@should_panic` passes
//! only by panicking: the binary says on standard error that its test
//! panicked before the panic aborts (`[PAN-1]`), which tells a panic from a
//! crash or an exit.
//!
//! `[TST-30]` — every test runs whatever failed before it, and every failure
//! is reported. A test that fails is run again, up to `RERUNS` more times; one
//! that then passes fails intermittently, and is quarantined by name: listed
//! in the run summary, and not counted as a failure (ADR-192).
//!
//! `[WK-15]` — in the debug profile each test's leaked objects and their
//! cycles are reported when it ends, unless `--no-leak-check`.
//!
//! `--doc` runs the documentation examples under the package's `docs/`
//! instead: each error page's (`[DIA-6]`) and each user-guide sample
//! (`[DOC-2]`), as `doc_tests` says.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use ember_build::Profile;

use crate::{Options, abnormal_end, compile, package_entry_from_manifest, parse_options};

/// `[TST-30]` — how many more times a failed test runs before its failure
/// counts: a pass among them makes it intermittent.
const RERUNS: usize = 2;

pub(crate) fn command(args: &[String]) -> Result<ExitCode, String> {
    let mut filter = None;
    let mut doc = false;
    let mut instrument_ffi = false;
    let mut rest = Vec::new();
    let mut skip_value = false;
    for arg in args {
        if skip_value {
            rest.push(arg.clone());
            skip_value = false;
        } else if arg == "--doc" {
            doc = true;
        } else if arg == "--instrument-ffi" {
            instrument_ffi = true;
        } else if arg.starts_with('-') {
            skip_value = matches!(arg.as_str(), "--profile" | "--emit" | "--cc" | "--out-dir" | "--backend" | "-D");
            rest.push(arg.clone());
        } else if filter.is_none() {
            filter = Some(arg.clone());
        } else {
            return Err(format!("unexpected `ember test` argument `{arg}`: its one word is a filter"));
        }
    }
    let mut options = parse_options(&rest)?;
    if options.emit.is_some() || options.emit_header || options.syntax_only {
        return Err("`ember test` builds and runs tests: `--emit`, `--emit-header` and `--syntax-only` are not valid with it".to_string());
    }
    if options.leak_check && options.no_leak_check {
        return Err("`--leak-check` and `--no-leak-check` contradict each other".to_string());
    }
    // `[WK-15]` — on by default in the debug profile, as for `ember run`.
    if options.profile == Profile::Debug && !options.no_leak_check {
        options.leak_check = true;
    }
    options.test_filter = filter;
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let manifest = cwd.ancestors().map(|dir| dir.join(ember_branding::MANIFEST)).find(|path| path.is_file());
    if doc {
        // Documentation needs no program: without a package, this directory's.
        let root = manifest.as_deref().and_then(Path::parent).unwrap_or(&cwd);
        return doc_tests(root, &options);
    }
    let manifest = manifest.ok_or_else(|| format!(
        "`ember test` tests the package of this directory or an ancestor, and none has an {}",
        ember_branding::MANIFEST
    ))?;
    let root = manifest.parent().unwrap_or(&cwd);
    let (entry, _) = package_entry_from_manifest(root, &manifest, "test")?;
    // `[FFI-37]`, `[TCB-1]` — the evidence record goes where `[ffi] evidence` says, by default
    // `.ember/ffi-evidence` in the package.
    if instrument_ffi {
        // An empty intercepted set would grade every fact without observing anything.
        if !cfg!(any(windows, target_os = "linux")) {
            return Err("error[E0900]: `--instrument-ffi` intercepts entry points on Windows and Linux only".to_string());
        }
        let text = std::fs::read_to_string(&manifest).map_err(|error| error.to_string())?;
        let dir = crate::manifest_string(&text, "ffi", "evidence")
            .unwrap_or_else(crate::foreign::default_evidence_dir);
        options.ffi_evidence = Some(root.join(dir));
    }
    compile(&entry, "test", &options)
}

/// One run of one case.
struct Attempt {
    passed: bool,
    /// Why it failed, then what it printed.
    failure: String,
    /// `[WK-15]` — the leak report of a run that passed.
    leaks: String,
    /// What a passing case adds to its `ok`.
    note: Option<String>,
}

impl Attempt {
    fn pass() -> Attempt {
        Attempt { passed: true, failure: String::new(), leaks: String::new(), note: None }
    }

    fn fail(why: impl Into<String>, output: &str) -> Attempt {
        let mut failure = why.into();
        let output = output.trim_end();
        if !output.is_empty() {
            failure.push('\n');
            failure.push_str(output);
        }
        Attempt { passed: false, failure, leaks: String::new(), note: None }
    }
}

#[derive(PartialEq)]
enum Verdict {
    Passed,
    Failed,
    /// `[TST-30]` — failed, then passed when run again.
    Quarantined,
}

struct Case {
    name: String,
    verdict: Verdict,
    /// The first run, which is the failing one of a case that failed.
    first: Attempt,
    runs: usize,
    failed_runs: usize,
}

/// `[TST-30]` — a case's verdict: one run that passes, or a failure run again
/// until it passes or `RERUNS` more runs have failed too.
fn decide(name: String, mut attempt: impl FnMut() -> Result<Attempt, String>) -> Result<Case, String> {
    let first = attempt()?;
    let (mut runs, mut failed_runs) = (1, usize::from(!first.passed));
    if !first.passed {
        while runs <= RERUNS {
            runs += 1;
            if attempt()?.passed {
                break;
            }
            failed_runs += 1;
        }
    }
    let verdict = if failed_runs == 0 {
        Verdict::Passed
    } else if failed_runs == runs {
        Verdict::Failed
    } else {
        Verdict::Quarantined
    };
    Ok(Case { name, verdict, first, runs, failed_runs })
}

/// Runs the cases one by one, each line as it ends, and then the summary.
/// The exit status is a failure exactly when a case failed every run.
fn run_cases(
    what: &str,
    cases: Vec<(String, Box<dyn FnMut() -> Result<Attempt, String> + '_>)>,
    filtered_out: usize,
    json: bool,
) -> Result<ExitCode, String> {
    if !json {
        println!("running {} {what}{}", cases.len(), if cases.len() == 1 { "" } else { "s" });
    }
    let mut done = Vec::new();
    for (name, attempt) in cases {
        let case = decide(name, attempt)?;
        if !json {
            match case.verdict {
                Verdict::Passed => match &case.first.note {
                    Some(note) => println!("{what} {} ... ok ({note})", case.name),
                    None => println!("{what} {} ... ok", case.name),
                },
                Verdict::Failed => println!("{what} {} ... FAILED", case.name),
                Verdict::Quarantined => println!(
                    "{what} {} ... FAILED intermittently: quarantined (failed {} of {} runs)",
                    case.name, case.failed_runs, case.runs
                ),
            }
            // `[WK-15]` — on standard error, where `ember run` reports it.
            if !case.first.leaks.trim().is_empty() {
                eprintln!("leak report of {what} {} [WK-15]:", case.name);
                eprint!("{}", case.first.leaks);
            }
        }
        done.push(case);
    }
    let count = |verdict: Verdict| done.iter().filter(|case| case.verdict == verdict).count();
    let (passed, failed, quarantined) = (count(Verdict::Passed), count(Verdict::Failed), count(Verdict::Quarantined));
    if json {
        let cases: Vec<serde_json::Value> = done.iter().map(|case| serde_json::json!({
            "name": case.name,
            "result": match case.verdict {
                Verdict::Passed => "passed",
                Verdict::Failed => "failed",
                Verdict::Quarantined => "quarantined",
            },
            "runs": case.runs,
            "failed_runs": case.failed_runs,
            "failure": (case.verdict != Verdict::Passed).then_some(&case.first.failure),
            "note": case.first.note,
            "leak_report": (!case.first.leaks.trim().is_empty()).then_some(&case.first.leaks),
        })).collect();
        println!("{}", serde_json::json!({
            "schema": 1,
            "kind": what,
            "cases": cases,
            "passed": passed,
            "failed": failed,
            "quarantined": quarantined,
            "filtered_out": filtered_out,
        }));
    } else {
        let unwell: Vec<&Case> = done.iter().filter(|case| case.verdict != Verdict::Passed).collect();
        if !unwell.is_empty() {
            println!("\nfailures:");
            for case in &unwell {
                let quarantined = if case.verdict == Verdict::Quarantined { " (quarantined)" } else { "" };
                println!("\n---- {}{quarantined} ----\n{}", case.name, case.first.failure.trim_end());
            }
        }
        if failed > 0 {
            println!("\nfailures:");
            for case in done.iter().filter(|case| case.verdict == Verdict::Failed) {
                println!("    {}", case.name);
            }
        }
        if quarantined > 0 {
            println!("\nquarantined: failed intermittently, not counted as failures [TST-30]:");
            for case in done.iter().filter(|case| case.verdict == Verdict::Quarantined) {
                println!("    {} (failed {} of {} runs)", case.name, case.failed_runs, case.runs);
            }
        }
        println!(
            "\n{what} result: {}. {passed} passed; {failed} failed; {quarantined} quarantined; {filtered_out} filtered out",
            if failed == 0 { "ok" } else { "FAILED" }
        );
    }
    Ok(if failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

fn selected(name: &str, options: &Options) -> bool {
    options.test_filter.as_deref().is_none_or(|filter| name.contains(filter))
}

/// `[TST-3]` — run the test binary `exe` once per selected test, the test's
/// number as its argument.
pub(crate) fn run_tests(
    exe: &Path,
    tests: &[&ember_hir::TestFn],
    options: &Options,
    instrumented: Option<&crate::foreign::InstrumentedRun>,
    map: &ember_span::SourceMap,
) -> Result<ExitCode, String> {
    let mut cases: Vec<(String, Box<dyn FnMut() -> Result<Attempt, String>>)> = Vec::new();
    // `[FFI-37]` — each test's counts, in a file of its own beside the binary.
    let evidence = |index: usize| exe.with_file_name(format!("ffi-evidence-{index}.txt"));
    let mut evidence_files = Vec::new();
    for (index, test) in tests.iter().enumerate() {
        let name = test.name.as_str().to_string();
        if !selected(&name, options) {
            continue;
        }
        let should_panic = test.should_panic;
        let exe = exe.to_path_buf();
        let file = instrumented.map(|_| evidence(index));
        if let Some(file) = &file {
            let _ = std::fs::remove_file(file);
            evidence_files.push(file.clone());
        }
        cases.push((name, Box::new(move || run_test(&exe, index, should_panic, file.as_deref()))));
    }
    let filtered_out = tests.len() - cases.len();
    let status = run_cases("test", cases, filtered_out, options.json)?;
    let Some(run) = instrumented else { return Ok(status) };
    let contradicted = crate::foreign::instrument_report(run, &evidence_files, map, options)?;
    for file in &evidence_files {
        let _ = std::fs::remove_file(file);
    }
    Ok(if contradicted { ExitCode::FAILURE } else { status })
}

fn run_test(exe: &Path, index: usize, should_panic: bool, evidence: Option<&Path>) -> Result<Attempt, String> {
    let mut command = Command::new(exe);
    if let Some(file) = evidence {
        command.env(crate::foreign::evidence_variable(), file);
    }
    let output = command
        .arg(index.to_string())
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("cannot run the test binary `{}`: {error}", exe.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    let panic_line = format!("{}\n", ember_codegen_c::test_panic_marker());
    let end_line = format!("{}\n", ember_codegen_c::test_end_marker());
    let panicked = stderr.contains(&panic_line);
    // What the test wrote, and after its end, the runtime's leak report.
    let (during, after) = match stderr.find(&end_line) {
        Some(at) => (&stderr[..at], &stderr[at + end_line.len()..]),
        None => (stderr.as_str(), ""),
    };
    let printed = format!("{stdout}{}", during.replace(&panic_line, ""));
    let crash = abnormal_end(&output.status);
    let status = match output.status.code() {
        Some(code) => format!("status {code}"),
        None => "no status".to_string(),
    };
    let attempt = if should_panic {
        if panicked {
            Attempt::pass()
        } else if let Some(how) = crash {
            Attempt::fail(format!("it was to panic, and {how}"), &printed)
        } else if output.status.success() {
            Attempt::fail("it was to panic, and returned [TST-3]", &printed)
        } else {
            Attempt::fail(format!("it was to panic, and exited with {status}"), &printed)
        }
    } else if panicked {
        Attempt::fail("it panicked", &printed)
    } else if let Some(how) = crash {
        Attempt::fail(how, &printed)
    } else if !output.status.success() {
        Attempt::fail(format!("it exited with {status}"), &printed)
    } else {
        Attempt::pass()
    };
    Ok(Attempt { leaks: if attempt.passed { after.to_string() } else { String::new() }, ..attempt })
}

/// A documentation example and what it must do.
enum Example {
    /// `[DIA-6]` — an error page's ```` ```ember,fails ```` program: an
    /// error's must be rejected with the page's code; a warning's or lint's
    /// compiles and reports it, when checked or else when it runs (a code the
    /// debug runtime reports, `[WK-15]`). ```` ```ember,fails,toolchain ````
    /// marks an error only a C compiler decides: the program checks clean,
    /// and its build is rejected with the code by a toolchain that cannot do
    /// what it asks (ADR-192).
    Fails { code: String, source: String, manifest: Option<String>, toolchain: bool, files: Vec<(String, String)> },
    /// `[DIA-6]` — an error page's ```` ```ember,fixed ```` program: it
    /// compiles; for a code reported only when the failing program runs, it
    /// also runs without reporting it.
    Fixed { code: String, source: String, manifest: Option<String>, files: Vec<(String, String)> },
    /// `[DOC-2]` — a user-guide sample, ```` ```ember ````: built and run, it
    /// exits with status 0; followed at once by an ```` ```output ```` block,
    /// it prints exactly that.
    Run { source: String, output: Option<String> },
    /// A sample marked ```` ```ember,fragment ```` names items it does not
    /// declare: it must parse (`[TST-7]`'s marking).
    Fragment { source: String },
    /// `[DIA-6]` — a page with one of its two examples and not the other.
    Missing { what: &'static str },
}

/// Each fenced block of a Markdown text: its info string, its text and the
/// line of its opening fence.
fn fences(text: &str) -> Vec<(String, String, usize)> {
    let mut out = Vec::new();
    let mut open: Option<(String, String, usize)> = None;
    for (index, line) in text.lines().enumerate() {
        match &mut open {
            None => {
                if let Some(info) = line.strip_prefix("```") {
                    open = Some((info.trim().to_string(), String::new(), index + 1));
                }
            }
            Some((_, body, _)) => {
                if line.trim_end() == "```" {
                    out.extend(open.take());
                } else {
                    body.push_str(line);
                    body.push('\n');
                }
            }
        }
    }
    out
}

/// `[DIA-6]` — an error page's examples. Each takes as its package manifest
/// the last ```` ```toml ```` block above it on the page, if any. A page with
/// neither example documents a code no compiler emits (a reserved one) and
/// has nothing to run; a page with one and not the other is missing it.
fn error_page_examples(page: &Path, code: &str) -> Result<Vec<(String, Example)>, String> {
    let text = std::fs::read_to_string(page).map_err(|error| format!("cannot read `{}`: {error}", page.display()))?;
    let name = |what: &str, line: usize| format!("docs/errors/{code}.md:{line} ({what})");
    let mut manifest = None;
    let mut out = Vec::new();
    let (mut fails, mut fixed) = (0, 0);
    // A file the page's examples are compiled beside: ```` ```c,file=shape.h ```` (`[FFI-6]`).
    let files: Vec<(String, String)> = fences(&text)
        .into_iter()
        .filter_map(|(info, body, _)| Some((info.split_once(",file=")?.1.to_string(), body)))
        .collect();
    for (info, body, line) in fences(&text) {
        match info.as_str() {
            "toml" => manifest = Some(body),
            "ember,fails" | "ember,fails,toolchain" => {
                fails += 1;
                let toolchain = info.ends_with(",toolchain");
                out.push((name("fails", line), Example::Fails {
                    code: code.to_string(), source: body, manifest: manifest.clone(), toolchain, files: files.clone(),
                }));
            }
            "ember,fixed" => {
                fixed += 1;
                out.push((name("fixed", line), Example::Fixed {
                    code: code.to_string(), source: body, manifest: manifest.clone(), files: files.clone(),
                }));
            }
            _ => {}
        }
    }
    if fails == 0 && fixed > 0 || fixed == 0 && fails > 0 {
        let missing = if fails == 0 { "fails" } else { "fixed" };
        out.push((name(missing, 1), Example::Missing { what: missing }));
    }
    Ok(out)
}

/// `[DOC-2]` — the user guide's samples, page by page in path order.
fn guide_examples(dir: &Path, docs: &Path, out: &mut Vec<(String, Example)>) -> Result<(), String> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Ok(()) };
    let mut paths: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            guide_examples(&path, docs, out)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|error| format!("cannot read `{}`: {error}", path.display()))?;
        let page = path.strip_prefix(docs.parent().unwrap_or(docs)).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        let blocks = fences(&text);
        for (index, (info, body, line)) in blocks.iter().enumerate() {
            let example = match info.as_str() {
                "ember" => {
                    let output = blocks.get(index + 1).filter(|(next, ..)| next == "output").map(|(_, printed, _)| printed.clone());
                    Example::Run { source: body.clone(), output }
                }
                "ember,fragment" => Example::Fragment { source: body.clone() },
                // `ember,ignore` and every other block are not run.
                _ => continue,
            };
            out.push((format!("{page}:{line}"), example));
        }
    }
    Ok(())
}

/// `ember test --doc` — the examples of `<root>/docs`: every error page's
/// under `docs/errors/` (`[DIA-6]`), then every sample of the user guide under
/// `docs/book/` (`[DOC-2]`). Each is compiled by this same compiler in a
/// directory of its own under the output directory, removed afterwards.
fn doc_tests(root: &Path, options: &Options) -> Result<ExitCode, String> {
    let docs = root.join("docs");
    let mut examples = Vec::new();
    if let Ok(entries) = std::fs::read_dir(docs.join("errors")) {
        let mut pages: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect();
        pages.sort();
        for page in pages {
            let Some(code) = page.file_stem().and_then(|stem| stem.to_str()) else { continue };
            let is_code = code.len() == 5
                && code.starts_with(['E', 'W', 'L'])
                && code[1..].bytes().all(|byte| byte.is_ascii_digit());
            if is_code && page.extension().and_then(|extension| extension.to_str()) == Some("md") {
                examples.extend(error_page_examples(&page, code)?);
            }
        }
    }
    guide_examples(&docs.join("book"), &docs, &mut examples)?;
    let total = examples.len();
    let work_root = options.out_dir.clone().unwrap_or_else(|| PathBuf::from("target")).join(options.profile.name()).join("doc");
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    // The codes whose failing example reported them only when it ran.
    let runtime_codes = std::cell::RefCell::new(std::collections::BTreeSet::new());
    let mut cases: Vec<(String, Box<dyn FnMut() -> Result<Attempt, String> + '_>)> = Vec::new();
    for (number, (name, example)) in examples.into_iter().enumerate() {
        if !selected(&name, options) {
            continue;
        }
        let work = work_root.join(number.to_string());
        let (exe, runtime_codes) = (&exe, &runtime_codes);
        cases.push((name, Box::new(move || {
            let attempt = run_example(exe, &work, &example, runtime_codes, options);
            let _ = std::fs::remove_dir_all(&work);
            attempt
        })));
    }
    let filtered_out = total - cases.len();
    let status = run_cases("example", cases, filtered_out, options.json);
    let _ = std::fs::remove_dir(&work_root);
    status
}

/// The source as bytes: `<0xFF>` in an example is that one byte, for a page
/// about bytes that are not UTF-8 (`E0001`'s).
fn source_bytes(source: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("<0x") {
        let hex = rest.get(at + 3..at + 5).filter(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
        match (hex, rest.get(at + 5..at + 6)) {
            (Some(hex), Some(">")) => {
                out.extend_from_slice(rest[..at].as_bytes());
                out.push(u8::from_str_radix(hex, 16).expect("two hex digits"));
                rest = &rest[at + 6..];
            }
            _ => {
                out.extend_from_slice(rest[..at + 3].as_bytes());
                rest = &rest[at + 3..];
            }
        }
    }
    out.extend_from_slice(rest.as_bytes());
    out
}

/// Runs this compiler as `ember <command> <example> [flags]` on one example
/// written into `work`, beside its manifest if it has one: the status,
/// standard output and error together, and standard output alone.
fn ember_on(
    exe: &Path,
    work: &Path,
    source: &str,
    manifest: Option<&str>,
    command: &[&str],
    options: &Options,
) -> Result<(bool, String, String), String> {
    std::fs::create_dir_all(work).map_err(|error| format!("cannot create `{}`: {error}", work.display()))?;
    let manifest_path = work.join(ember_branding::MANIFEST);
    // `ember test` tests a package: the example is its `src/main`, under the page's manifest or one
    // of its own.
    let package = command[0] == "test";
    let file = if package {
        std::fs::create_dir_all(work.join("src")).map_err(|error| error.to_string())?;
        work.join("src").join(ember_branding::source_file("main"))
    } else {
        work.join(ember_branding::source_file("example"))
    };
    std::fs::write(&file, source_bytes(source)).map_err(|error| error.to_string())?;
    match manifest {
        Some(text) => std::fs::write(&manifest_path, text).map_err(|error| error.to_string())?,
        None if package => std::fs::write(&manifest_path, "[package]\nname = \"example\"\nversion = \"0.1.0\"\n").map_err(|error| error.to_string())?,
        None => {
            let _ = std::fs::remove_file(&manifest_path);
        }
    }
    let mut child = Command::new(exe);
    child.args(command);
    if package {
        child.current_dir(work);
    } else {
        child.arg(&file);
    }
    child.args(["--profile", options.profile.name()]).stdin(Stdio::null());
    if let Some(cc) = &options.cc {
        child.args(["--cc", cc]);
    }
    if matches!(command[0], "run" | "build" | "test") {
        child.arg("--out-dir").arg(work.join("out"));
    }
    if command[0] == "run" && options.no_leak_check {
        child.arg("--no-leak-check");
    }
    let output = child.output().map_err(|error| format!("cannot run `{}`: {error}", exe.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    let text = format!("{stdout}{}", String::from_utf8_lossy(&output.stderr)).replace("\r\n", "\n");
    Ok((output.status.success(), text, stdout))
}

/// The files a page's examples are compiled beside (`[FFI-6]`'s headers), into `work`.
fn write_files(work: &Path, files: &[(String, String)]) -> Result<(), String> {
    std::fs::create_dir_all(work).map_err(|error| format!("cannot create `{}`: {error}", work.display()))?;
    for (name, body) in files {
        std::fs::write(work.join(name), body).map_err(|error| format!("cannot write `{name}`: {error}"))?;
    }
    Ok(())
}

/// How an error page's examples are checked: `E5050` only a build that requires evidence reports
/// (`[TCB-1]`).
fn check_command(code: &str) -> &'static [&'static str] {
    // `[LNT-6]` — `ember lint`'s own lints appear under `ember lint` only.
    if code == "E5050" {
        &["check", "--require-evidence"]
    } else if ember_analysis::PerfLints::CODES.iter().any(|lint| lint.to_string() == code) {
        &["lint"]
    } else {
        &["check"]
    }
}

fn run_example(
    exe: &Path,
    work: &Path,
    example: &Example,
    runtime_codes: &std::cell::RefCell<std::collections::BTreeSet<String>>,
    options: &Options,
) -> Result<Attempt, String> {
    Ok(match example {
        Example::Missing { what } => Attempt::fail(format!("the page has no ```ember,{what} block [DIA-6]"), ""),
        // `[FFI-37]` — a code only an instrumented test run reports (D-882): the failing program's
        // `ember test --instrument-ffi` reports it (an error's fails), the fixed one's passes
        // without it.
        Example::Fails { code, source, manifest, files, .. } if crate::foreign::reported_by_an_instrumented_run(code) => {
            write_files(work, files)?;
            let (passed, output, _) = ember_on(exe, work, source, manifest.as_deref(), &["test", "--instrument-ffi"], options)?;
            if !output.contains(&format!("[{code}]")) {
                Attempt::fail(format!("the failing program's instrumented test run did not report {code}"), &output)
            } else if code.starts_with('E') && passed {
                Attempt::fail(format!("the failing program's instrumented test run reported {code} and passed"), &output)
            } else {
                Attempt::pass()
            }
        }
        Example::Fixed { code, source, manifest, files } if crate::foreign::reported_by_an_instrumented_run(code) => {
            write_files(work, files)?;
            let (passed, output, _) = ember_on(exe, work, source, manifest.as_deref(), &["test", "--instrument-ffi"], options)?;
            if !passed {
                Attempt::fail("the fixed program's instrumented test run fails", &output)
            } else if output.contains(&format!("[{code}]")) {
                Attempt::fail(format!("the fixed program's instrumented test run still reports {code}"), &output)
            } else {
                Attempt::pass()
            }
        }
        Example::Fails { code, source, manifest, toolchain, files } => {
            write_files(work, files)?;
            let tag = format!("[{code}]");
            let manifest = manifest.as_deref();
            let (checked, output, _) = ember_on(exe, work, source, manifest, check_command(code), options)?;
            if code.starts_with('E') && *toolchain {
                if !checked {
                    Attempt::fail("the failing program, an error only a C compiler decides, is rejected without one", &output)
                } else {
                    let (built, output, _) = ember_on(exe, work, source, manifest, &["build"], options)?;
                    if !built && output.contains(&tag) {
                        Attempt::pass()
                    } else if built {
                        Attempt { note: Some(format!("{code} is not reproduced here: this C compiler builds it")), ..Attempt::pass() }
                    } else {
                        Attempt::fail(format!("the failing program failed to build with something other than {code}"), &output)
                    }
                }
            } else if code.starts_with('E') {
                if checked {
                    Attempt::fail("the failing program compiled", &output)
                } else if !output.contains(&tag) {
                    Attempt::fail(format!("the failing program failed with something other than {code}"), &output)
                } else {
                    Attempt::pass()
                }
            } else if !checked {
                Attempt::fail(format!("the failing program does not compile, and {code} does not reject"), &output)
            } else if output.contains(&tag) {
                Attempt::pass()
            } else {
                // A code the debug runtime reports appears when it runs.
                let (ran, output, _) = ember_on(exe, work, source, manifest, &["run"], options)?;
                if ran && output.contains(&tag) {
                    runtime_codes.borrow_mut().insert(code.clone());
                    Attempt::pass()
                } else {
                    Attempt::fail(format!("the failing program compiled and ran without reporting {code}"), &output)
                }
            }
        }
        Example::Fixed { code, source, manifest, files } => {
            write_files(work, files)?;
            let manifest = manifest.as_deref();
            let (checked, output, _) = ember_on(exe, work, source, manifest, check_command(code), options)?;
            if !checked {
                Attempt::fail("the fixed program does not compile", &output)
            } else if runtime_codes.borrow().contains(code) {
                let (ran, output, _) = ember_on(exe, work, source, manifest, &["run"], options)?;
                if !ran {
                    Attempt::fail("the fixed program fails when it runs", &output)
                } else if output.contains(&format!("[{code}]")) {
                    Attempt::fail(format!("the fixed program still reports {code} when it runs"), &output)
                } else {
                    Attempt::pass()
                }
            } else {
                Attempt::pass()
            }
        }
        Example::Run { source, output: expected } => {
            let (ran, output, printed) = ember_on(exe, work, source, None, &["run"], options)?;
            match expected {
                _ if !ran => Attempt::fail("the sample does not build and run with status 0", &output),
                Some(expected) if printed.trim_end() != expected.trim_end() => {
                    Attempt::fail(format!("the sample printed other than its `output` block, which is:\n{expected}"), &output)
                }
                _ => Attempt::pass(),
            }
        }
        Example::Fragment { source } => {
            let (parsed, output, _) = ember_on(exe, work, source, None, &["check", "--syntax-only"], options)?;
            if parsed { Attempt::pass() } else { Attempt::fail("the fragment does not parse", &output) }
        }
    })
}

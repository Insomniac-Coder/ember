//! `[TST-5]` — a scripted debugger session on the host's debugger: `cdb` for a Windows program
//! (its PDB), `gdb` on Linux, `lldb` on macOS. It stops at a breakpoint set by an Ember file and
//! line, steps, and reads locals by their Ember names (the C names them so, and `#line` maps its
//! lines to the Ember file's). A host where it cannot run is reported as debug-unverified, which
//! a waiver in `tests/debugger/waivers.json` must record: CI sets `EMBER_DEBUGGER_REQUIRED`, and
//! there a host with no debugger and no waiver fails.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

const PROGRAM: &str = "\
fn total(values: Array[int]) -> int:
    sum = 0
    for v in values:
        sum += v
    return sum

fn main():
    numbers = [3, 4, 5]
    result = total(numbers)
    println(result)
";

/// The line of `sum += v`, where the session stops.
const STOP_LINE: u32 = 4;

#[derive(Clone, Copy, Debug)]
enum Debugger {
    Cdb,
    Gdb,
    Lldb,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("the workspace root").to_path_buf()
}

/// This host as the waivers name it: `windows-x86_64`, `linux-x86_64`, `macos-aarch64`.
fn host() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// The debugger this host's programs are debugged with, if it is installed.
fn find_debugger() -> Option<(Debugger, PathBuf)> {
    let on_path = |name: &str| -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path).map(|dir| dir.join(name)).find(|candidate| candidate.is_file())
    };
    if cfg!(windows) {
        let kits = ["ProgramFiles(x86)", "ProgramFiles"]
            .iter()
            .filter_map(|var| std::env::var_os(var))
            .map(|dir| PathBuf::from(dir).join("Windows Kits").join("10").join("Debuggers").join("x64").join("cdb.exe"));
        return on_path("cdb.exe").into_iter().chain(kits).find(|path| path.is_file()).map(|path| (Debugger::Cdb, path));
    }
    if cfg!(target_os = "macos") {
        return on_path("lldb").map(|path| (Debugger::Lldb, path));
    }
    on_path("gdb").map(|path| (Debugger::Gdb, path))
}

/// How to install this host's debugger, for the message when it is missing.
fn install_hint() -> &'static str {
    if cfg!(windows) {
        "install the Debugging Tools for Windows (the Windows SDK's \"Debugging Tools for Windows\" feature, e.g. `winget install Microsoft.WindowsSDK.10.0.22621` choosing it), which put cdb.exe in C:\\Program Files (x86)\\Windows Kits\\10\\Debuggers\\x64"
    } else if cfg!(target_os = "macos") {
        "install the Xcode command line tools (`xcode-select --install`), which bring lldb"
    } else {
        "install gdb (`sudo apt install gdb`)"
    }
}

/// The waiver recorded for this host, if any.
fn waiver() -> Option<String> {
    let text = std::fs::read_to_string(workspace_root().join("tests").join("debugger").join("waivers.json")).ok()?;
    let waivers: serde_json::Value = serde_json::from_str(&text).expect("tests/debugger/waivers.json is JSON");
    waivers.get(host()).and_then(|reason| reason.as_str()).map(str::to_string)
}

/// The folder of `program`, where cdb is to find its program database: never a symbol cache's,
/// since a build whose code is the same as an earlier one's has its program database's signature
/// (`/Brepro`), and a cached one of another build would be taken for it.
fn beside(program: &Path) -> &Path {
    program.parent().expect("the program is in a folder")
}

/// The session's commands, and its output: stop at the line, list the locals, go on to the
/// line's second turn and print `sum`, then step.
fn session(debugger: Debugger, path: &Path, program: &Path, source: &str) -> String {
    let output = match debugger {
        Debugger::Cdb => {
            // A line whose code has several ranges (clang-cl starts one at each branch, so a line
            // with a check has more than one; cl joins them) is "Ambiguous" to cdb, which lists each
            // as `Matched: module!function+offset`: as a cdb user does, the breakpoint goes on the
            // first, the line's start.
            let line = format!("`{source}:{STOP_LINE}`");
            let probe = Command::new(path).args(["-lines", "-y"]).arg(beside(program)).args(["-c", &format!(".lines -e; bu {line}; q")]).arg(program).output().expect("the debugger runs");
            let probe = String::from_utf8_lossy(&probe.stdout).into_owned();
            let location = match probe.contains("Ambiguous symbol error") {
                true => probe.lines().find_map(|text| text.strip_prefix("Matched: ")).and_then(|text| text.split_whitespace().next()).map(str::to_string).unwrap_or(line),
                false => line,
            };
            // `l+t`: step by source line; `.lines -e`: source lines from the PDB; the breakpoint is
            // resolved as the program loads (`bu`).
            let script = format!(
                ".lines -e; l+t; l+s; bu {location}; g; .printf \"MARK%d\\n\", 1; .frame; dv; .printf \"MARK%d\\n\", 2; g; dv sum; .printf \"MARK%d\\n\", 3; p; .frame; .printf \"MARK%d\\n\", 4; q"
            );
            Command::new(path).args(["-lines", "-y"]).arg(beside(program)).args(["-c", &script]).arg(program).output()
        }
        Debugger::Gdb => Command::new(path)
            .args(["-batch", "-nx", "-iex", "set auto-load safe-path /"])
            .args(["-ex", &format!("break {source}:{STOP_LINE}"), "-ex", "run"])
            .args(["-ex", "printf \"MARK%d\\n\", 1", "-ex", "info line *$pc", "-ex", "info args", "-ex", "info locals"])
            .args(["-ex", "printf \"MARK%d\\n\", 2", "-ex", "continue", "-ex", "print sum"])
            .args(["-ex", "printf \"MARK%d\\n\", 3", "-ex", "next", "-ex", "info line *$pc", "-ex", "printf \"MARK%d\\n\", 4"])
            .arg(program)
            .output(),
        Debugger::Lldb => Command::new(path)
            .args(["-b", "-o", &format!("breakpoint set --file {source} --line {STOP_LINE}"), "-o", "run"])
            .args(["-o", "script print('MARK%d' % 1)", "-o", "frame info", "-o", "frame variable"])
            .args(["-o", "script print('MARK%d' % 2)", "-o", "continue", "-o", "frame variable sum"])
            .args(["-o", "script print('MARK%d' % 3)", "-o", "next", "-o", "frame info", "-o", "script print('MARK%d' % 4)"])
            .arg(program)
            .output(),
    }
    .expect("the debugger runs");
    format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)).replace("\r\n", "\n")
}

/// The part of the session's output between two markers it echoed.
fn between<'a>(output: &'a str, from: &str, to: &str) -> &'a str {
    let start = output.find(from).unwrap_or_else(|| panic!("the session never reached {from}:\n{output}"));
    let rest = &output[start + from.len()..];
    &rest[..rest.find(to).unwrap_or(rest.len())]
}

/// The line of `result = total(numbers)`, where `numbers` is set.
const NUMBERS_LINE: u32 = 9;

/// `[CG-C-9]` (ODR-178) — with the build's `.natvis`, the debugger shows a collection held directly
/// in a local as its elements, of their type: with the C backend the local is its type's own C
/// struct, which the `.natvis` names; with the LLVM backend (`[CG-LL-3]`) its type's own debug
/// type, `Array<i64>`, which the `.natvis` matches as every `Array`. Visual Studio's engine
/// (`cdb`'s `dx`) only: the GDB and LLDB scripts are read by
/// `build_writes_debugger_visualisers_beside_the_program` and its LLVM twin.
#[test]
fn a_collection_held_in_a_local_shows_its_elements() {
    let Some((Debugger::Cdb, path)) = find_debugger() else {
        eprintln!("the natvis session needs cdb (Windows): {}", install_hint());
        return;
    };
    for backend in ["c", "llvm"] {
        collection_shows_its_elements(&path, backend);
    }
}

fn collection_shows_its_elements(path: &Path, backend: &str) {
    let dir = temporary_directory("debugger-natvis");
    let source_name = format!("count.{}", ember_branding::SOURCE_EXT);
    std::fs::write(dir.join(&source_name), PROGRAM).expect("the program is written");
    let out = dir.join("out");
    let built = Command::new(EMBER)
        .args(["build", &source_name, "--backend", backend, "--profile", "debug", "--out-dir", &out.to_string_lossy()])
        .current_dir(&dir)
        .output()
        .expect("ember runs");
    assert!(built.status.success(), "{backend}: the build failed:\n{}", String::from_utf8_lossy(&built.stderr));
    let bin = out.join("debug").join("bin");
    let program = bin.join(format!("count{}", std::env::consts::EXE_SUFFIX));
    let natvis = bin.join("count.natvis");
    assert!(natvis.is_file(), "{backend}: no .natvis beside the program in {}", bin.display());
    let script = format!(
        ".lines -e; .nvload {}; bu `{source_name}:{NUMBERS_LINE}`; g; .printf \"MARK%d\\n\", 1; dx -r1 numbers; .printf \"MARK%d\\n\", 2; q",
        natvis.display()
    );
    let output = Command::new(&path)
        .args(["-lines", "-y"])
        .arg(beside(&program))
        .args(["-c", &script])
        .arg(&program)
        .output()
        .expect("the debugger runs");
    let output = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)).replace("\r\n", "\n");
    let shown = between(&output, "MARK1", "MARK2");
    for (index, value) in [(0, 3), (1, 4), (2, 5)] {
        let element = shown.lines().any(|line| line.trim_start().starts_with(&format!("[{index}]")) && line.contains(&format!(": {value}")));
        assert!(element, "{backend}: cdb does not show `numbers[{index}]` as {value}:\n{shown}\n(the whole session:\n{output})");
    }
    // Its length, which cdb writes in hexadecimal.
    assert!(shown.contains("numbers") && shown.contains("[0x3]"), "{backend}: cdb does not show `numbers`'s length:\n{shown}");
}

#[test]
fn a_debugger_session_stops_at_an_ember_line_steps_and_shows_locals_by_their_ember_names() {
    debugger_session("c");
}

/// `[TST-5]`, `[CG-LL-3]` — the same session on a program the LLVM backend built: its debug
/// information (CodeView or DWARF) names the Ember file, lines and locals.
#[test]
fn the_llvm_backends_program_gives_the_same_debugger_session() {
    debugger_session("llvm");
}

fn debugger_session(backend: &str) {
    let Some((debugger, path)) = find_debugger() else {
        let required = std::env::var_os("EMBER_DEBUGGER_REQUIRED").is_some();
        match waiver() {
            Some(reason) => eprintln!("debug-unverified on {}: {reason} (recorded waiver)", host()),
            None if required => panic!("[TST-5] no debugger on {} and no waiver recorded for it: {}", host(), install_hint()),
            None => eprintln!("debug-unverified on this machine ({}): no debugger found; {}", host(), install_hint()),
        }
        return;
    };
    let dir = temporary_directory("debugger");
    let source_name = format!("count.{}", ember_branding::SOURCE_EXT);
    std::fs::write(dir.join(&source_name), PROGRAM).expect("the program is written");
    let out = dir.join("out");
    let built = Command::new(EMBER)
        .args(["build", &source_name, "--profile", "debug", "--backend", backend, "--out-dir", &out.to_string_lossy()])
        .current_dir(&dir)
        .output()
        .expect("ember runs");
    assert!(built.status.success(), "the build failed:\n{}", String::from_utf8_lossy(&built.stderr));
    let program = out.join("debug").join("bin").join(format!("count{}", std::env::consts::EXE_SUFFIX));
    let output = session(debugger, &path, &program, &source_name);

    // Stopped at the Ember line, in the Ember file.
    let stopped = between(&output, "MARK1", "MARK2");
    let at_line = |text: &str, line: u32| match debugger {
        Debugger::Cdb => text.contains(&format!("{source_name} @ {line}")),
        Debugger::Gdb => text.contains(&format!("Line {line} of \"")) && text.contains(&source_name),
        Debugger::Lldb => text.contains(&format!("{source_name}:{line}")),
    };
    assert!(at_line(stopped, STOP_LINE), "{debugger:?} did not stop at {source_name}:{STOP_LINE}:\n{output}");
    // The locals, by the names the Ember program gives them, `sum` still 0.
    for name in ["sum", "v", "values"] {
        let named = stopped.lines().any(|line| line.trim_start().starts_with(&format!("{name} ")) || line.contains(&format!(" {name} =")) || line.trim_start().starts_with(&format!("{name} =")));
        assert!(named, "{debugger:?} shows no local `{name}`:\n{stopped}");
    }
    let zero = match debugger {
        Debugger::Cdb => stopped.contains("sum = 0n0"),
        Debugger::Gdb | Debugger::Lldb => stopped.lines().any(|line| line.trim_start().starts_with("sum = 0") || line.contains("sum = 0")),
    };
    assert!(zero, "{debugger:?} shows `sum` other than 0 at the first stop:\n{stopped}");
    // The second turn: `sum` is the first element.
    let again = between(&output, "MARK2", "MARK3");
    let three = match debugger {
        Debugger::Cdb => again.contains("sum = 0n3"),
        Debugger::Gdb => again.contains("$1 = 3"),
        Debugger::Lldb => again.contains("sum = 3"),
    };
    assert!(three, "{debugger:?} shows `sum` other than 3 on the second turn:\n{again}");
    // A step goes on to an Ember line of the loop.
    let stepped = between(&output, "MARK3", "MARK4");
    assert!((2..=5).any(|line| at_line(stepped, line)), "{debugger:?}'s step left the loop's lines:\n{output}");
}

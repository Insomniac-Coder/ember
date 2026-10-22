//! `[CLI-21]` — `ember run --interp` runs a program on the evaluator: no C compiler is needed,
//! compile-time values included; the console and files work; a foreign call or a thread is
//! `E6010`; and what it prints is what the compiled program prints (every run test of the suite
//! runs both ways: `assert_the_evaluator_agrees` in `milestones.rs` and `annotations.py`,
//! `[IMP-11]`).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2).expect("the workspace root").to_path_buf()
}

struct Run {
    stdout: String,
    stderr: String,
    exit: i32,
}

/// `ember` with `args` in `dir`, standard input `input`, and the C compiler variable set to
/// `cc` when given.
fn ember(args: &[&str], dir: &Path, input: &str, cc: Option<&str>) -> Run {
    let mut command = Command::new(EMBER);
    command.args(args).current_dir(dir).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(cc) = cc {
        command.env(ember_branding::cc_var(), cc);
    }
    let mut child = command.spawn().expect("ember starts");
    child.stdin.take().expect("piped stdin").write_all(input.as_bytes()).expect("stdin is written");
    let output = child.wait_with_output().expect("ember finishes");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

const CONSOLE_AND_FILES: &str = r#"import std.fs

fn table(n: int) -> Array[int]:
    out: Array[int] = []
    for i in range(n):
        out.push(i * i)
    return out

static SQUARES: Array[int] = table(5)

fn main(args: Array[String]):
    path = args[1].as_str()
    name = input("name? ")
    match fs.write(path, f"hello {name}\n".as_bytes()):
        Ok(_):
            pass
        Err(e):
            println(f"cannot write: {e}")
    match fs.read_to_string(path):
        Ok(text):
            print(text)
        Err(e):
            println(f"cannot read: {e}")
    println(SQUARES, 7 // 2, 2.0 / 3.0, f"{255:x}")
    _ = fs.remove(path)
"#;

/// `[CLI-21]` — with no C compiler (the variable naming one that does not exist), the evaluator
/// still builds and runs the program: its compile-time value, its console and its files, exactly
/// as the compiled program does with a real C compiler.
#[test]
fn the_evaluator_runs_a_program_with_no_c_compiler_and_its_console_and_files_work() {
    let dir = temporary_directory("interp-console");
    let source = dir.join(format!("console.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&source, CONSOLE_AND_FILES).expect("the program is written");
    let file = dir.join("greeting.txt");
    let (source, file) = (source.to_string_lossy().into_owned(), file.to_string_lossy().into_owned());
    let out = dir.join("out").to_string_lossy().into_owned();
    let expected = "name? hello ann\n[0, 1, 4, 9, 16] 3 0.6666666666666666 ff\n";

    let interpreted = ember(&["run", "--interp", &source, "--out-dir", &out, "--", &file], &dir, "ann\n", Some("no-such-c-compiler"));
    assert_eq!(interpreted.exit, 0, "the evaluator failed:\n{}", interpreted.stderr);
    assert_eq!(interpreted.stdout, expected, "stderr:\n{}", interpreted.stderr);
    assert!(!Path::new(&file).exists(), "the program removed its file");

    let refused = ember(&["run", &source, "--out-dir", &out, "--", &file], &dir, "ann\n", Some("no-such-c-compiler"));
    assert_ne!(refused.exit, 0, "a compiled run needs the C compiler it names");

    let compiled = ember(&["run", &source, "--out-dir", &out, "--", &file], &dir, "ann\n", None);
    assert_eq!(compiled.exit, 0, "the compiled run failed:\n{}", compiled.stderr);
    assert_eq!(compiled.stdout, interpreted.stdout, "both ways print the same");
}

/// `[CLI-21]`, `[CT-2]` — what compile time cannot run stops the evaluator with `E6010`, at the
/// call: a foreign function, and a thread.
#[test]
fn a_foreign_call_and_a_thread_are_e6010_on_the_evaluator() {
    let dir = temporary_directory("interp-refused");
    let out = dir.join("out").to_string_lossy().into_owned();
    let cases = [
        (
            "foreign",
            "unsafe extern \"C\":\n    @ffi(effects=[])\n    safe fn abs(x: i32) -> i32\n\nfn main():\n    println(\"before\")\n    println(abs(-3))\n",
            "the foreign function `abs`",
        ),
        (
            "thread",
            "import std.thread\n\nfn main():\n    println(\"before\")\n    h = thread.spawn(owned fn() => 1)\n    println(h.join())\n",
            "a thread",
        ),
    ];
    for (name, program, what) in cases {
        let source = dir.join(format!("{name}.{}", ember_branding::SOURCE_EXT));
        std::fs::write(&source, program).expect("the program is written");
        let run = ember(&["run", "--interp", &source.to_string_lossy(), "--out-dir", &out], &dir, "", None);
        assert_ne!(run.exit, 0, "{name}: the evaluator ran what it cannot");
        assert_eq!(run.stdout, "before\n", "{name}: the program ran up to the call");
        assert!(run.stderr.contains("error[E6010]"), "{name}:\n{}", run.stderr);
        assert!(run.stderr.contains(&format!("{what} cannot run on the evaluator")), "{name}:\n{}", run.stderr);
    }
}

/// `[CLI-21]`, `[IMP-11]` — a panic, its exit status and its message are the compiled
/// program's, and so is a compile-time panic's `E6004`.
#[test]
fn a_panic_on_the_evaluator_is_the_compiled_programs() {
    let root = workspace_root();
    let run_fail = format!("tests/run-fail/{}", first_file(&root.join("tests/run-fail")));
    let out = temporary_directory("interp-panic");
    let out = out.to_string_lossy().into_owned();
    let compiled = ember(&["run", &run_fail, "--out-dir", &out], &root, "", None);
    let interpreted = ember(&["run", "--interp", &run_fail, "--out-dir", &out], &root, "", None);
    assert_ne!(compiled.exit, 0);
    assert_eq!(interpreted.exit, compiled.exit, "{}", interpreted.stderr);
    assert_eq!(interpreted.stdout, compiled.stdout);
    let panic = |stderr: &str| stderr.lines().find(|line| line.starts_with("panic")).map(str::to_string);
    assert_eq!(panic(&interpreted.stderr), panic(&compiled.stderr));

    let dir = temporary_directory("interp-comptime-panic");
    let source = dir.join(format!("ct.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&source, "fn f() -> int:\n    xs = [1]\n    return xs[3]\n\nconst X: int = f()\n\nfn main():\n    println(X)\n")
        .expect("the program is written");
    let source = source.to_string_lossy().into_owned();
    let out = dir.join("out").to_string_lossy().into_owned();
    let compiled = ember(&["run", &source, "--out-dir", &out], &dir, "", None);
    let interpreted = ember(&["run", "--interp", &source, "--out-dir", &out], &dir, "", Some("no-such-c-compiler"));
    let errors = |stderr: &str| stderr.lines().filter(|line| line.starts_with("error[") || line.trim_start().starts_with("= note")).map(str::to_string).collect::<Vec<_>>();
    assert!(compiled.stderr.contains("error[E6004]"), "{}", compiled.stderr);
    assert_eq!(errors(&interpreted.stderr), errors(&compiled.stderr), "\n{}", interpreted.stderr);
}

fn first_file(dir: &Path) -> String {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .expect("the directory is readable")
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name().to_string_lossy().into_owned()))
        .filter(|name| name.ends_with(&format!(".{}", ember_branding::SOURCE_EXT)))
        .collect();
    names.sort();
    names.remove(0)
}

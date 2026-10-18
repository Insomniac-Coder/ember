//! Hot reload (Annex B): the build mode's manifest and command line, and reloads of a running
//! program. A program is run only through these helpers, never by hand, so no window opens.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

use ember_branding::MANIFEST;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("the workspace root").to_path_buf()
}

struct Run {
    stdout: String,
    stderr: String,
    exit: i32,
}

fn ember(args: &[&str], cwd: &Path) -> Run {
    let output = Command::new(EMBER).args(args).current_dir(cwd).output().expect("the ember binary runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// A package directory holding `manifest` and one source file, `main.em`.
fn package(label: &str, manifest: &str, source: &str) -> (test_directories::TemporaryDirectory, PathBuf) {
    let directory = temporary_directory(label);
    std::fs::write(directory.join(MANIFEST), manifest).expect("manifest is writable");
    let file = directory.join(ember_branding::source_file("main"));
    std::fs::write(&file, source).expect("source is writable");
    (directory, file)
}

/// `[MAN-7]` — `[build] reload` is `"opt-in"`, `"all"`, `"bodies"` or `"none"`: each of the four is
/// accepted, and any other word is `E9031`, pointing at the manifest and naming the four.
#[test]
fn reload_takes_four_values() {
    let root = workspace_root();
    for value in ["opt-in", "all", "bodies", "none"] {
        let (_package, file) = package("reload-value", &format!("[build]\nreload = \"{value}\"\n"), "fn main():\n    pass\n");
        let checked = ember(&["check", &file.to_string_lossy()], &root);
        assert_eq!(checked.exit, 0, "[MAN-7] `reload = \"{value}\"` was refused:\n{}", checked.stderr);
    }
    let (_package, file) = package("reload-bad-value", "[build]\nreload = \"sometimes\"\n", "fn main():\n    pass\n");
    let checked = ember(&["check", &file.to_string_lossy()], &root);
    assert_ne!(checked.exit, 0, "[MAN-7] `reload = \"sometimes\"` was accepted");
    assert!(checked.stderr.contains("error[E9031]") && checked.stderr.contains("`sometimes` is not a `reload` value"), "{}", checked.stderr);
    assert!(checked.stderr.contains("\"opt-in\", \"all\", \"bodies\" or \"none\""), "{}", checked.stderr);
    assert!(checked.stderr.contains(MANIFEST), "the error does not point at the manifest:\n{}", checked.stderr);
}

/// `[PRF-2]`, `[MAN-7]` — hot reload is a build mode admitted in `debug` and `release`: a package
/// asking for it is checked in both, and `shipping` refuses it with `E9033`, whether the manifest or
/// `ember build --reload` asks.
#[test]
fn reload_is_forbidden_in_shipping() {
    let root = workspace_root();
    let (_package, file) = package("reload-shipping", "[build]\nreload = \"all\"\n", "fn main():\n    pass\n");
    let file = file.to_string_lossy().into_owned();
    for profile in ["debug", "release"] {
        let checked = ember(&["check", "--profile", profile, &file], &root);
        assert_eq!(checked.exit, 0, "[PRF-2] reload was refused in `{profile}`:\n{}", checked.stderr);
    }
    let checked = ember(&["check", "--profile", "shipping", &file], &root);
    assert_ne!(checked.exit, 0, "[PRF-2] reload was accepted in `shipping`");
    assert!(checked.stderr.contains("error[E9033]") && checked.stderr.contains("`shipping` does not admit"), "{}", checked.stderr);
    assert!(checked.stderr.contains(MANIFEST), "the error does not point at the manifest:\n{}", checked.stderr);

    let (_plain, plain) = package("reload-flag-shipping", "[package]\nname = \"p\"\n", "fn main():\n    pass\n");
    let built = ember(&["build", "--reload", "--profile", "shipping", &plain.to_string_lossy()], &root);
    assert_ne!(built.exit, 0, "[PRF-2] `ember build --reload` was accepted in `shipping`");
    assert!(built.stderr.contains("error[E9033]"), "{}", built.stderr);
}

/// A reloadable program's build directory: `<out>/debug/bin`, holding the host, the shared runtime
/// and the images.
fn bin(out: &Path) -> PathBuf {
    out.join("debug").join("bin")
}

fn host(out: &Path) -> PathBuf {
    bin(out).join(if cfg!(windows) { "main.exe" } else { "main" })
}

/// Run a built host, as the harness runs every program: its output, never a window.
fn run_host(out: &Path) -> Run {
    let mut command = Command::new(host(out));
    command.current_dir(out);
    let output = ember_build::output_started(&mut command).expect("the host runs");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// Build `source` as the first image of a reloadable program in `out`, then each of `edits` as
/// the next image (`ember build --reload`), and run the host: it starts with the first image and
/// its polls find the last one announced.
fn reload_run(label: &str, manifest: &str, source: &str, edits: &[&str]) -> (Run, Vec<Run>) {
    let root = workspace_root();
    let (package, file) = package(label, manifest, source);
    let out = package.join("target");
    let out_arg = out.to_string_lossy().into_owned();
    let file_arg = file.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "the first build failed:\n{}", built.stderr);
    let mut reloads = Vec::new();
    for edit in edits {
        std::fs::write(&file, edit).expect("the edit is writable");
        reloads.push(ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root));
    }
    let run = run_host(&out);
    drop(package);
    (run, reloads)
}

const RELOAD_ALL: &str = "[build]\nreload = \"all\"\n";

/// `[HR-2]`, `[HR-5]`, `[HR-6]`, `[HR-30]`, `[HR-33]` — an edited body is running after the next
/// poll: the program starts with its first image, `std.hot.checkpoint()` polls, the runtime loads
/// the image `ember build --reload` announced and repoints the thunks, and the next call of `step`
/// runs the new body while `main`'s own frame runs on. What was computed before stays.
#[test]
fn an_edited_body_runs_after_the_next_checkpoint() {
    let v1 = "import std.hot\n\nfn step(i: int) -> int:\n    println(f\"v1 step {i}\")\n    return i * 10\n\n\
              fn main():\n    total = 0\n    for i in 0..3:\n        total += step(i)\n        hot.checkpoint()\n    println(f\"total {total}\")\n";
    let v2 = v1.replace("v1 step", "v2 step").replace("i * 10", "i * 100");
    let (run, reloads) = reload_run("reload-body", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "the reload build failed:\n{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "v1 step 0\nv2 step 1\nv2 step 2\ntotal 300\n", "{}", run.stderr);
}

/// `[HR-29]` — with reload on, one shared runtime serves every image in the process: a reloadable
/// package whose manifest asks to link the runtime statically is `E9036`.
#[test]
fn a_reloadable_package_shares_the_runtime() {
    let root = workspace_root();
    let (_package, file) =
        package("reload-static-runtime", "[build]\nreload = \"all\"\nruntime = \"static\"\n", "fn main():\n    pass\n");
    let checked = ember(&["check", &file.to_string_lossy()], &root);
    assert_ne!(checked.exit, 0, "[HR-29] a reloadable package linked its own runtime");
    assert!(checked.stderr.contains("error[E9036]") && checked.stderr.contains("its own copy of the runtime"), "{}", checked.stderr);
    // Without reload, a static runtime is the default and is accepted.
    let (_plain, plain) = package("static-runtime", "[build]\nreload = \"none\"\nruntime = \"static\"\n", "fn main():\n    pass\n");
    let checked = ember(&["check", &plain.to_string_lossy()], &root);
    assert_eq!(checked.exit, 0, "{}", checked.stderr);
}

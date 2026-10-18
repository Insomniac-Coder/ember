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

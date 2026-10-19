//! Packages: `ember new`, the manifest's `language` key, and what `ember --version` says.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!("{}-package-{}-{}",
        ember_branding::CLI_NAME, std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

struct Run {
    stdout: String,
    stderr: String,
    ok: bool,
}

fn ember(args: &[&str], cwd: &Path) -> Run {
    let output = ember_build::output_started(Command::new(EMBER).args(args).current_dir(cwd)).expect("ember starts");
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        ok: output.status.success(),
    }
}

/// `[TOOL-4]` — the first hour: `ember new hello`, `cd hello` and `ember run` print `hello, world`,
/// three typed commands well inside five minutes. `ember --version` names the compiler, the
/// language, the backend, and the C compiler and linker it resolved. `ember new` also makes a
/// library and a shared library that check and build, and never writes into a directory that is
/// not empty.
#[test]
fn the_first_hour_is_new_cd_and_run() {
    let dir = scratch();
    let started = std::time::Instant::now();
    let mut typed = 0;
    let created = ember(&["new", "hello"], &dir);
    typed += 1;
    assert!(created.ok, "{}", created.stderr);
    let hello = dir.join("hello");
    typed += 1; // cd hello
    let ran = ember(&["run"], &hello);
    typed += 1;
    assert!(ran.ok, "{}", ran.stderr);
    assert_eq!(ran.stdout, "hello, world\n");
    assert!(typed <= 6 && started.elapsed().as_secs() < 300, "{typed} commands, {:?}", started.elapsed());

    let version = ember(&["--version"], &dir);
    assert!(version.ok);
    let lines: Vec<&str> = version.stdout.lines().collect();
    assert_eq!(lines[0], format!("{} {}", ember_branding::CLI_NAME, env!("CARGO_PKG_VERSION")));
    assert_eq!(lines[1], format!("language: {}", ember_parser::LANGUAGE_VERSION));
    assert_eq!(lines[2], "backend: c");
    assert!(lines[3].starts_with("C compiler: ") && !lines[3].contains("none found"), "{}", version.stdout);
    assert!(lines[4].starts_with("linker: ") && lines[4].len() > "linker: ".len(), "{}", version.stdout);

    let again = ember(&["new", "hello"], &dir);
    assert!(!again.ok && again.stderr.contains("already exists and is not empty"), "{}", again.stderr);
    assert_eq!(std::fs::read_to_string(hello.join("src").join(ember_branding::source_file("main"))).unwrap(),
        "println(\"hello, world\")\n");

    assert!(ember(&["new", "shapes", "--lib"], &dir).ok);
    let checked = ember(&["check"], &dir.join("shapes"));
    assert!(checked.ok, "{}", checked.stderr);
    assert!(ember(&["new", "plugin", "--cdylib"], &dir).ok);
    let built = ember(&["build"], &dir.join("plugin"));
    assert!(built.ok, "{}", built.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[VER-2]` — a package opts into its language with the manifest's `language` key; before 1.0
/// there is one language (`[VER-8]`), so the key names the current version or the manifest is
/// invalid (`E9001`, naming the current one).
#[test]
fn the_manifest_names_the_language() {
    let dir = scratch();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src").join(ember_branding::source_file("main")), "println(\"ok\")\n").unwrap();
    let manifest = |language: &str| format!("[package]\nname = \"app\"\nlanguage = \"{language}\"\nkind = \"bin\"\n");
    std::fs::write(dir.join(ember_branding::MANIFEST), manifest(ember_parser::LANGUAGE_VERSION)).unwrap();
    let current = ember(&["check"], &dir);
    assert!(current.ok, "{}", current.stderr);
    std::fs::write(dir.join(ember_branding::MANIFEST), manifest("0.8")).unwrap();
    let older = ember(&["check"], &dir);
    assert!(!older.ok);
    assert!(older.stderr.contains("error[E9001]: `language = \"0.8\"` is not the current language"), "{}", older.stderr);
    assert!(older.stderr.contains(&format!("write `language = \"{}\"`", ember_parser::LANGUAGE_VERSION)), "{}", older.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

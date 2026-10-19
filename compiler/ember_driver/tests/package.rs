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

/// A package in `dir`: its manifest and its sources, each `(module name under src, text)`.
fn package(dir: &Path, manifest: &str, sources: &[(&str, &str)]) {
    for (module, text) in sources {
        let path = dir.join("src").join(ember_branding::source_file(module));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    std::fs::write(dir.join(ember_branding::MANIFEST), manifest).unwrap();
}

/// The packages of the dependency tests: `app` depends on `geometry` 2.1.0 and on `mesh`, which
/// depends on `geometry` 1.4.0; each `geometry` has a `shapes` module and a private helper of the
/// same name, and a `pub(package)` function.
fn dependency_tree(dir: &Path) {
    for (folder, version, answer) in [("geometry1", "1.4.0", 1), ("geometry2", "2.1.0", 2)] {
        package(&dir.join(folder), &format!("[package]\nname = \"geometry\"\nversion = \"{version}\"\nkind = \"lib\"\n"), &[
            ("lib", &format!("pub fn major() -> i32:\n    return {answer}\n\npub(package) fn inner() -> i32:\n    return {answer}\n")),
            ("shapes", &format!("pub struct Square:\n    pub side: i32\n\npub fn area(s: Square) -> i32:\n    return s.side * s.side * {answer}\n")),
        ]);
    }
    package(&dir.join("mesh"), "[package]\nname = \"mesh\"\nversion = \"0.3.0\"\nkind = \"lib\"\n\n[dependencies]\ngeometry = { path = \"../geometry1\" }\n", &[
        ("lib", "import geometry\nimport geometry.shapes\nimport util\n\npub fn geometry_major() -> i32:\n    return geometry.major() + util.zero()\n\npub fn area() -> i32:\n    return shapes.area(shapes.Square(3))\n"),
        ("util", "pub fn zero() -> i32:\n    return 0\n"),
    ]);
    package(&dir.join("app"), "[package]\nname = \"app\"\nversion = \"0.1.0\"\nkind = \"bin\"\n\n[dependencies]\ngeometry = { path = \"../geometry2\" }\n\n[dependencies.mesh]\npath = \"../mesh\"\n", &[
        ("main", "import geometry\nimport geometry.shapes\nimport mesh\n\nfn main():\n    println(geometry.major())\n    println(mesh.geometry_major())\n    println(shapes.area(shapes.Square(3)))\n    println(mesh.area())\n"),
    ]);
}

/// `[MOD-1]`, `[VER-5]` — a dependency's modules are its name followed by their path; two majors of
/// one package coexist in one build, each package seeing the one it depends on, and their symbols
/// differ in the C. `[MOD-2]`: a dependency's `pub(package)` function is not visible to its
/// dependant (`E1052`).
#[test]
fn two_majors_of_a_dependency_coexist() {
    let dir = scratch();
    dependency_tree(&dir);
    let app = dir.join("app");
    let ran = ember(&["run"], &app);
    assert!(ran.ok, "{}", ran.stderr);
    assert_eq!(ran.stdout, "2\n1\n18\n9\n");
    let c = std::fs::read_to_string(app.join("target/debug/c/main.c")).unwrap();
    // (Their small functions are inlined into `main`; the types and the unused functions remain.)
    let symbols = ["geometry.shapes.Square ", "geometry__1.shapes.Square ", "geometry.inner(", "geometry__1.inner("];
    for symbol in symbols.map(ember_branding::mangled) {
        assert!(c.contains(&symbol), "no `{symbol}` in the C");
    }
    std::fs::write(app.join("src").join(ember_branding::source_file("main")),
        "import geometry\n\nfn main():\n    println(geometry.inner())\n").unwrap();
    let hidden = ember(&["check"], &app);
    assert!(!hidden.ok && hidden.stderr.contains("error[E1052]"), "{}", hidden.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[MAN-2]` — a build with dependencies writes `ember.lock`: each dependency's name, version,
/// source and a hash of its contents. `--locked` fails when the lock would change (none yet, or a
/// dependency's contents changed) and passes when it would not; `ember update` rewrites it, and
/// `ember update <package>` takes only that package's entry anew.
#[test]
fn the_lock_records_dependencies_with_content_hashes() {
    let dir = scratch();
    dependency_tree(&dir);
    let app = dir.join("app");
    let lock = app.join(ember_branding::LOCK_FILE);
    let none_yet = ember(&["check", "--locked"], &app);
    assert!(!none_yet.ok && none_yet.stderr.contains("would change (there is none)"), "{}", none_yet.stderr);
    assert!(!lock.exists());
    let checked = ember(&["check"], &app);
    assert!(checked.ok, "{}", checked.stderr);
    let text = std::fs::read_to_string(&lock).unwrap();
    for entry in ["name = \"geometry\"\nversion = \"1.4.0\"\nsource = \"path+../geometry1\"\nhash = \"blake3:",
        "name = \"geometry\"\nversion = \"2.1.0\"\nsource = \"path+../geometry2\"\nhash = \"blake3:",
        "name = \"mesh\"\nversion = \"0.3.0\"", "dependencies = [\"geometry 1.4.0\"]"] {
        assert!(text.contains(entry), "the lock lacks `{entry}`:\n{text}");
    }
    assert!(ember(&["check", "--locked"], &app).ok);

    // A dependency's contents change: the lock would change.
    std::fs::write(dir.join("mesh/src").join(ember_branding::source_file("util")), "pub fn zero() -> i32:\n    return 0 * 1\n").unwrap();
    let changed = ember(&["check", "--locked"], &app);
    assert!(!changed.ok && changed.stderr.contains("the dependencies changed since it was written"), "{}", changed.stderr);
    assert_eq!(std::fs::read_to_string(&lock).unwrap(), text, "--locked wrote the lock");
    let geometry = ember(&["update", "geometry"], &app);
    assert!(geometry.ok, "{}", geometry.stderr);
    assert_eq!(std::fs::read_to_string(&lock).unwrap(), text, "updating geometry changed mesh's entry");
    let updated = ember(&["update"], &app);
    assert!(updated.ok, "{}", updated.stderr);
    assert_ne!(std::fs::read_to_string(&lock).unwrap(), text);
    assert!(ember(&["check", "--locked"], &app).ok);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[MAN-2]` — a `{ git, rev }` dependency is checked out at its revision; the lock records the
/// repository and the revision with the contents' hash, and a later commit on the repository
/// changes nothing until the manifest names it.
#[test]
fn a_git_dependency_is_checked_out_at_its_revision() {
    let dir = scratch();
    let repo = dir.join("repo");
    package(&repo, "[package]\nname = \"greet\"\nversion = \"1.0.0\"\nkind = \"lib\"\n",
        &[("lib", "pub fn word() -> i32:\n    return 1\n")]);
    let git = |args: &[&str]| {
        let output = Command::new("git").args(["-c", "user.name=ember", "-c", "user.email=ember@example.org", "-c", "core.autocrlf=false"])
            .args(args).current_dir(&repo).output().expect("git runs");
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };
    git(&["init", "--quiet"]);
    git(&["add", "-A"]);
    git(&["commit", "--quiet", "-m", "first"]);
    let first = git(&["rev-parse", "HEAD"]);
    std::fs::write(repo.join("src").join(ember_branding::source_file("lib")), "pub fn word() -> i32:\n    return 2\n").unwrap();
    git(&["commit", "--quiet", "-am", "second"]);
    let url = repo.to_string_lossy().replace('\\', "/");
    package(&dir.join("app"), &format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ngreet = {{ git = \"{url}\", rev = \"{first}\" }}\n"),
        &[("main", "import greet\n\nfn main():\n    println(greet.word())\n")]);
    let ran = ember(&["run"], &dir.join("app"));
    assert!(ran.ok, "{}", ran.stderr);
    assert_eq!(ran.stdout, "1\n", "the dependency was not checked out at its revision");
    let lock = std::fs::read_to_string(dir.join("app").join(ember_branding::LOCK_FILE)).unwrap();
    assert!(lock.contains(&format!("source = \"git+{url}#{first}\"\nhash = \"blake3:")), "{lock}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[VER-5]` (ODR-127) — `{ git, version }` takes the highest of the repository's `vMAJOR.MINOR.PATCH`
/// tags the requirement admits; `ember.lock` keeps that commit, so a newer tag changes nothing until
/// `ember update` meets the requirement anew; a requirement no tag meets is `E9001`, listing the tags.
#[test]
fn a_git_dependency_resolves_its_version_from_the_tags() {
    let dir = scratch();
    let repo = dir.join("repo");
    let release = |version: &str, word: i32| {
        package(&repo, &format!("[package]\nname = \"greet\"\nversion = \"{version}\"\nkind = \"lib\"\n"),
            &[("lib", &format!("pub fn word() -> i32:\n    return {word}\n"))]);
    };
    let git = |args: &[&str]| {
        let output = Command::new("git").args(["-c", "user.name=ember", "-c", "user.email=ember@example.org", "-c", "core.autocrlf=false"])
            .args(args).current_dir(&repo).output().expect("git runs");
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
    };
    release("1.2.0", 120);
    git(&["init", "--quiet"]);
    let tag = |version: &str, annotated: bool| {
        git(&["add", "-A"]);
        git(&["commit", "--quiet", "-m", version]);
        if annotated {
            git(&["tag", "-a", &format!("v{version}"), "-m", version]);
        } else {
            git(&["tag", &format!("v{version}")]);
        }
    };
    tag("1.2.0", false);
    release("1.3.1", 131);
    tag("1.3.1", true);
    release("2.0.0", 200);
    tag("2.0.0", false);
    let url = repo.to_string_lossy().replace('\\', "/");
    let app = |requirement: &str| {
        package(&dir.join("app"), &format!("[package]\nname = \"app\"\nversion = \"0.1.0\"\n\n[dependencies]\ngreet = {{ git = \"{url}\", version = \"{requirement}\" }}\n"),
            &[("main", "import greet\n\nfn main():\n    println(greet.word())\n")]);
    };
    app("1.2");
    let ran = ember(&["run"], &dir.join("app"));
    assert!(ran.ok && ran.stdout == "131\n", "the highest 1.x tag was not taken: {}{}", ran.stdout, ran.stderr);
    let lock = std::fs::read_to_string(dir.join("app").join(ember_branding::LOCK_FILE)).unwrap();
    assert!(lock.contains("version = \"1.3.1\"") && lock.contains(&format!("source = \"git+{url}#")), "{lock}");

    // A newer 1.x tag: the lock keeps 1.3.1 until `ember update`.
    release("1.4.0", 140);
    tag("1.4.0", false);
    let kept = ember(&["run"], &dir.join("app"));
    assert!(kept.ok && kept.stdout == "131\n", "the lock did not keep its commit: {}{}", kept.stdout, kept.stderr);
    let updated = ember(&["update"], &dir.join("app"));
    assert!(updated.ok, "{}", updated.stderr);
    let moved = ember(&["run"], &dir.join("app"));
    assert!(moved.ok && moved.stdout == "140\n", "`ember update` did not meet the requirement anew: {}{}", moved.stdout, moved.stderr);

    app("2");
    let major = ember(&["run"], &dir.join("app"));
    assert!(major.ok && major.stdout == "200\n", "{}{}", major.stdout, major.stderr);
    app("3.1");
    let none = ember(&["check"], &dir.join("app"));
    assert!(!none.ok && none.stderr.contains("error[E9001]: no version tag of")
        && none.stderr.contains("v1.2.0, v1.3.1, v1.4.0, v2.0.0"), "{}", none.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[MOD-4]` — packages that depend on each other in a cycle are `E1041`; `[VER-5]` a version that
/// is not `MAJOR.MINOR.PATCH` is `E9001`; a bare version requirement needs a registry (`E0900`).
#[test]
fn dependency_errors() {
    let dir = scratch();
    package(&dir.join("a"), "[package]\nname = \"a\"\nversion = \"1.0.0\"\n\n[dependencies]\nb = { path = \"../b\" }\n", &[("main", "println(1)\n")]);
    package(&dir.join("b"), "[package]\nname = \"b\"\nversion = \"1.0.0\"\nkind = \"lib\"\n\n[dependencies]\na = { path = \"../a\" }\n", &[("lib", "pub fn f() -> i32:\n    return 1\n")]);
    let cycle = ember(&["check"], &dir.join("a"));
    assert!(!cycle.ok && cycle.stderr.contains("error[E1041]") && cycle.stderr.contains("a -> b -> a"), "{}", cycle.stderr);

    package(&dir.join("b"), "[package]\nname = \"b\"\nversion = \"1.0\"\nkind = \"lib\"\n", &[]);
    let version = ember(&["check"], &dir.join("a"));
    assert!(!version.ok && version.stderr.contains("error[E9001]: package `b`'s version `1.0` is not a semantic version"), "{}", version.stderr);

    package(&dir.join("a"), "[package]\nname = \"a\"\nversion = \"1.0.0\"\n\n[dependencies]\nb = \"1.2\"\n", &[]);
    let registry = ember(&["check"], &dir.join("a"));
    assert!(!registry.ok && registry.stderr.contains("error[E0900]: dependency `b = \"1.2\"` needs a package registry"), "{}", registry.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[STD-6]` — `std` is layered (core, alloc, sync, io, ffi), each depending only on the ones
/// before it, and a package that declares `[package] layers = ["core"]` may use only core: its
/// options, results, integers, fixed arrays and math are there, an `Array` (the allocator) is not,
/// and with `alloc` added neither is the console. `[BLD-11]`: each is `E1021`, naming the layer
/// and the call chain; a package that leaves a layer out cannot depend on a package that uses it;
/// a layer listed without the ones it depends on is `E9001`.
#[test]
fn a_package_uses_only_its_layers() {
    let dir = scratch();
    let core_only = "[package]\nname = \"firmware\"\nversion = \"1.0.0\"\nkind = \"lib\"\nlayers = [\"core\"]\n";
    let fits = "pub fn checksum(data: ref [u8; 4]) -> Option[u32]:\n    total: u32 = 0\n    for byte in data:\n        total = total * 31 + byte as u32\n    if total == 0:\n        return None\n    return Some(max(total, 7))\n\npub fn clamp_to(x: i32, hi: i32) -> Result[i32, i32]:\n    if x > hi:\n        return Err(x - hi)\n    return Ok(x)\n";
    package(&dir.join("firmware"), core_only, &[("lib", fits)]);
    let checked = ember(&["check"], &dir.join("firmware"));
    assert!(checked.ok, "core-only code was refused: {}", checked.stderr);

    package(&dir.join("firmware"), core_only, &[("lib", &format!("{fits}\npub fn readings() -> i32:\n    values = [1, 2, 3]\n    values.push(4)\n    return values.len() as i32\n"))]);
    let alloc = ember(&["check"], &dir.join("firmware"));
    assert!(!alloc.ok && alloc.stderr.contains("error[E1021]: `readings` needs the `alloc` layer, which is not linked in this build"), "{}", alloc.stderr);
    assert!(alloc.stderr.contains("this package declares `[package] layers = [\"core\"]`"), "{}", alloc.stderr);

    let with_alloc = "[package]\nname = \"firmware\"\nversion = \"1.0.0\"\nkind = \"lib\"\nlayers = [\"core\", \"alloc\"]\n";
    package(&dir.join("firmware"), with_alloc, &[("lib", "pub fn report(x: i32) -> i32:\n    values = [x]\n    println(values.len())\n    return x\n")]);
    let io = ember(&["check"], &dir.join("firmware"));
    assert!(!io.ok && io.stderr.contains("error[E1021]: `report` needs the `io` layer"), "{}", io.stderr);
    assert!(!io.stderr.contains("needs the `alloc` layer"), "{}", io.stderr);

    // A dependency that uses every layer (it declares none) cannot serve a core-only package.
    package(&dir.join("helpers"), "[package]\nname = \"helpers\"\nversion = \"1.0.0\"\nkind = \"lib\"\n", &[("lib", "pub fn one() -> i32:\n    return 1\n")]);
    package(&dir.join("firmware"), &format!("{core_only}\n[dependencies]\nhelpers = {{ path = \"../helpers\" }}\n"), &[("lib", fits)]);
    let dependency = ember(&["check"], &dir.join("firmware"));
    assert!(!dependency.ok && dependency.stderr.contains("error[E1021]: `firmware` leaves out the `alloc` layer, which its dependency `helpers` uses"), "{}", dependency.stderr);
    package(&dir.join("helpers"), "[package]\nname = \"helpers\"\nversion = \"1.0.0\"\nkind = \"lib\"\nlayers = [\"core\"]\n", &[]);
    let fitting = ember(&["check"], &dir.join("firmware"));
    assert!(fitting.ok, "{}", fitting.stderr);

    package(&dir.join("firmware"), "[package]\nname = \"firmware\"\nversion = \"1.0.0\"\nkind = \"lib\"\nlayers = [\"core\", \"io\"]\n", &[]);
    let skipped = ember(&["check"], &dir.join("firmware"));
    assert!(!skipped.ok && skipped.stderr.contains("error[E9001]: the `io` layer depends on `alloc`"), "{}", skipped.stderr);
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

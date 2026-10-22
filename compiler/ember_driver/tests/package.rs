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

/// `[BLD-FFI-1b]` — with MSVC, the C++ project's runtime switch (and `_DEBUG`,
/// `_ITERATOR_DEBUG_LEVEL`) is inherited by every translation unit of the target: a project built
/// with `/MDd` makes a program whose own C uses the debug C runtime too, and none of the release
/// one. With any compiler, two projects that disagree are `E9020`, and a project whose flags pass a
/// response file, which may name its runtime, is `E9021`.
#[test]
fn the_cpp_project_s_runtime_is_inherited() {
    let probe = ember(&["--version"], &std::env::temp_dir());
    let header = "#pragma once\n#include <string>\ninline int length(int n) { return static_cast<int>(std::string(static_cast<std::size_t>(n), 'x').size()); }\n";
    let program = "import cpp \"engine.hpp\" with (project = \"engine\")\n\nfn main():\n    unsafe:  # SAFETY(ffi): a scalar call\n        \
        match cpp.length(5):\n            Ok(n):\n                println(n)\n            Err(error):\n                println(error.what)\n";
    // The MSVC runtime switch is cl's and clang-cl's: only their builds inherit it.
    if probe.stdout.lines().any(|line| line.starts_with("C compiler: msvc")) {
        let dir = scratch();
        package(&dir, "[package]\nname = \"engine_user\"\nversion = \"0.1.0\"\n\n[cpp.engine]\nflags = [\"/MDd\"]\n", &[("main", program)]);
        std::fs::write(dir.join("src").join("engine.hpp"), header).unwrap();
        let run = ember(&["run"], &dir);
        assert!(run.ok, "{}", run.stderr);
        assert_eq!(run.stdout.trim(), "5", "{}", run.stderr);
        let exe = walk(&dir).into_iter().find(|path| path.file_name().is_some_and(|name| name == "main.exe")).expect("the program");
        let image = String::from_utf8_lossy(&std::fs::read(&exe).unwrap()).to_lowercase();
        assert!(image.contains("ucrtbased.dll") && image.contains("vcruntime140d.dll"), "the program does not use the debug C runtime");
        assert!(!image.contains("api-ms-win-crt-") && !image.contains("vcruntime140.dll"), "the program's own C still uses the release C runtime");
        let _ = std::fs::remove_dir_all(&dir);
    }

    let dir = scratch();
    let two = "import cpp \"engine.hpp\" with (project = \"engine\")\nimport cpp \"tools.hpp\" with (project = \"tools\")\n\nfn main():\n    pass\n";
    package(&dir, "[package]\nname = \"two\"\nversion = \"0.1.0\"\n\n[cpp.engine]\nflags = [\"/MDd\"]\n\n[cpp.tools]\nflags = [\"/MD\"]\n", &[("main", two)]);
    std::fs::write(dir.join("src").join("engine.hpp"), header).unwrap();
    std::fs::write(dir.join("src").join("tools.hpp"), "#pragma once\ninline int twice(int x) noexcept { return 2 * x; }\n").unwrap();
    let built = ember(&["build"], &dir);
    assert!(!built.ok && built.stderr.contains("error[E9020]") && built.stderr.contains("`/MDd`") && built.stderr.contains("`/MD`"), "{}", built.stderr);
    let _ = std::fs::remove_dir_all(&dir);

    let dir = scratch();
    package(&dir, "[package]\nname = \"rsp\"\nversion = \"0.1.0\"\n\n[cpp.engine]\nflags = [\"@engine.rsp\"]\n", &[("main", program)]);
    std::fs::write(dir.join("src").join("engine.hpp"), header).unwrap();
    let built = ember(&["build"], &dir);
    assert!(!built.ok && built.stderr.contains("error[E9021]") && built.stderr.contains("`engine.rsp`"), "{}", built.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[BLD-FFI-4]` — with GCC or Clang, a C++ project whose flags ask for link-time optimisation
/// (`-flto`) has the program's own C compiled and linked with it too, so the link inlines the
/// calls into its thunks like any other call: the program runs, and (where `nm` reads it) the
/// thunk's symbol is gone from it. MSVC's `release` builds both with `/GL` already.
#[test]
fn the_cpp_project_s_link_time_optimisation_reaches_the_program() {
    let probe = ember(&["--version"], &std::env::temp_dir());
    if probe.stdout.lines().any(|line| line.starts_with("C compiler: msvc")) {
        return;
    }
    let dir = scratch();
    let program = "import cpp \"engine.hpp\" with (project = \"engine\")\n\nfn main():\n    unsafe:  # SAFETY(ffi): a scalar call\n        println(cpp.twice(21))\n";
    package(&dir, "[package]\nname = \"lto\"\nversion = \"0.1.0\"\n\n[cpp.engine]\nflags = [\"-flto\"]\n", &[("main", program)]);
    std::fs::write(dir.join("src").join("engine.hpp"), "#pragma once\ninline int twice(int x) noexcept { return x * 2; }\n").unwrap();
    let run = ember(&["run", "--profile", "release"], &dir);
    assert!(run.ok, "{}", run.stderr);
    assert_eq!(run.stdout.trim(), "42", "{}", run.stderr);
    let program = walk(&dir).into_iter().find(|path| path.file_stem().is_some_and(|stem| stem == "main") && path.parent().is_some_and(|parent| parent.ends_with("bin"))).expect("the program");
    if let Ok(listed) = Command::new("nm").arg(&program).output() && listed.status.success() {
        let thunk = format!("{}_cpp_cpp_h_engine_0", ember_branding::SYMBOL_PREFIX);
        let symbols = String::from_utf8_lossy(&listed.stdout);
        assert!(!symbols.lines().any(|line| line.ends_with(&format!(" T {thunk}"))), "the thunk was not inlined across the boundary:\n{symbols}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[CXX-1]` — the C++ annex's corpus (`tests/cxx-corpus`): nlohmann/json 3.11.3, a third-party
/// header-only library, imported whole, with each declaration's outcome recorded (`ember bind
/// --outcomes`: by value, opaque, behind an overlay, or refused with its diagnostic) and its thunk
/// file committed, both per platform. In every configuration — the release and debug runtimes,
/// RTTI on and off — under the C++ compiler the suite runs with (CI runs MSVC, clang-cl, GCC and
/// Clang), the import gives the recorded outcomes and thunks, and a program using the library
/// builds and prints what it should. Where the header defines a macro may follow the configuration
/// (`JSON_HAS_STATIC_RTTI`); what each declaration becomes may not. Two configurations that
/// disagree in one target are `E9020`, never two bindings: a class whose layout follows RTTI, and
/// the runtimes.
#[test]
fn the_cpp_corpus_binds_as_recorded() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/cxx-corpus");
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let probe = ember(&["--version"], &std::env::temp_dir());
    // cl and clang-cl take MSVC's switches; GCC and Clang theirs (libstdc++'s debug mode is their
    // debug runtime, and `-U_GLIBCXX_DEBUG` names the release one).
    let msvc = probe.stdout.lines().any(|line| line.starts_with("C compiler: msvc"));
    let configurations: [&[&str]; 4] = if msvc {
        [&["/MD", "/GR"], &["/MDd", "/GR"], &["/MD", "/GR-"], &["/MDd", "/GR-"]]
    } else {
        [&["-U_GLIBCXX_DEBUG"], &["-D_GLIBCXX_DEBUG"], &["-U_GLIBCXX_DEBUG", "-fno-rtti"], &["-D_GLIBCXX_DEBUG", "-fno-rtti"]]
    };
    let read = |path: &Path| std::fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read `{}`: {error}", path.display())).replace("\r\n", "\n");
    // The first line two texts differ on, and how to write the record again when a change is meant.
    let same = |found: &str, recorded: &str, what: &str| {
        if let Some((line, (found, recorded))) = found.lines().zip(recorded.lines()).enumerate().find(|(_, (found, recorded))| found != recorded) {
            panic!("{what}, line {}: `{found}`, recorded `{recorded}` (tests/cxx-corpus/README.md says how to record a meant change)", line + 1);
        }
        assert_eq!(found.lines().count(), recorded.lines().count(), "{what}: {} lines, recorded {}", found.lines().count(), recorded.lines().count());
    };
    let unplaced = |text: &str| text.lines().map(|line| line.rsplit_once(" (").map_or(line, |(kept, _)| kept).to_string()).collect::<Vec<_>>().join("\n");
    let entry = corpus.join("nlohmann_json");
    let third_party = corpus.join("third_party").to_string_lossy().replace('\\', "/");
    let outcomes = read(&entry.join(format!("nlohmann_json.{platform}.outcomes")));
    let thunks = read(&entry.join(format!("nlohmann_json.{platform}.thunks.cpp")));
    let program = corpus.join("program");
    let (main, api, printed) = (read(&program.join("main.em")), read(&program.join("json_api.hpp")), read(&program.join("expected.txt")));
    let dir = scratch();
    for (index, flags) in configurations.iter().enumerate() {
        let shown = flags.join(" ");
        let mut bind = vec!["bind", "nlohmann_json.hpp", "-I", &third_party, "--instantiate", "nlohmann::basic_json<>"];
        for flag in *flags {
            bind.extend(["--flag", flag]);
        }
        let found = ember(&[bind.as_slice(), &["--outcomes"]].concat(), &entry);
        assert!(found.ok, "`{shown}`: {}", found.stderr);
        if index == 0 {
            same(&found.stdout, &outcomes, &format!("the outcomes with `{shown}`"));
        } else {
            same(&unplaced(&found.stdout), &unplaced(&outcomes), &format!("the outcomes with `{shown}`"));
        }
        let out = dir.join(format!("thunks{index}.cpp")).to_string_lossy().into_owned();
        let emitted = ember(&[bind.as_slice(), &["--emit-cpp", "--out", &out]].concat(), &entry);
        assert!(emitted.ok, "`{shown}`: {}", emitted.stderr);
        same(&read(Path::new(&out)), &thunks, &format!("the thunks with `{shown}`"));
        // The program, built and run in this configuration.
        let built = dir.join(format!("program{index}"));
        let quoted: Vec<String> = flags.iter().map(|flag| format!("\"{flag}\"")).collect();
        let manifest = format!("[package]\nname = \"corpus\"\nversion = \"0.1.0\"\n\n[cpp.json]\ninclude_paths = [\"{third_party}\"]\nflags = [{}]\n", quoted.join(", "));
        package(&built, &manifest, &[("main", &main)]);
        std::fs::write(built.join("src").join("json_api.hpp"), &api).unwrap();
        let run = ember(&["run"], &built);
        assert!(run.ok, "`{shown}`: it printed
{}
{}", run.stdout, run.stderr.lines().filter(|line| !line.contains("W5001") && !line.contains("W5002")).collect::<Vec<_>>().join("
"));
        assert_eq!(run.stdout, printed, "`{shown}`: {}", run.stderr);
    }

    // Two configurations in one target: a class whose layout follows RTTI…
    let (with, without) = if msvc { ("/GR", "/GR-") } else { ("-frtti", "-fno-rtti") };
    let shape = "#pragma once\nstruct shape {\n    int sides;\n#if (defined(_MSC_VER) && defined(_CPPRTTI)) || (!defined(_MSC_VER) && defined(__GXX_RTTI))\n    int tagged;\n#endif\n};\n";
    let two = dir.join("two");
    let program = "import cpp \"one.hpp\" with (project = \"with\")\nimport cpp \"two.hpp\" with (project = \"without\") as two\n\nfn main():\n    pass\n";
    package(&two, &format!("[package]\nname = \"two\"\nversion = \"0.1.0\"\n\n[cpp.with]\nflags = [\"{with}\"]\n\n[cpp.without]\nflags = [\"{without}\"]\n"), &[("main", program)]);
    std::fs::write(two.join("src").join("shape.hpp"), shape).unwrap();
    std::fs::write(two.join("src").join("one.hpp"), "#pragma once\n#include \"shape.hpp\"\ninline int first(const shape* s) noexcept { return s->sides; }\n").unwrap();
    std::fs::write(two.join("src").join("two.hpp"), "#pragma once\n#include \"shape.hpp\"\ninline int second(const shape* s) noexcept { return s->sides; }\n").unwrap();
    let checked = ember(&["check"], &two);
    assert!(!checked.ok && checked.stderr.contains("error[E9020]") && checked.stderr.contains("their C++ configurations disagree"), "{}", checked.stderr);
    // Each configuration as the importer parsed with it: `/GR-` is clang-cl's `-fno-rtti-data`.
    assert!(checked.stderr.contains(if msvc { "-fno-rtti-data`" } else { "`-fno-rtti`" }), "{}", checked.stderr);

    // …and the corpus itself under two runtimes.
    let (release, debug) = if msvc { ("/MD", "/MDd") } else { ("-U_GLIBCXX_DEBUG", "-D_GLIBCXX_DEBUG") };
    let runtimes = dir.join("runtimes");
    let program = "import cpp \"nlohmann_json.hpp\" with (project = \"release\")\nimport cpp \"nlohmann_json.hpp\" with (project = \"debug\") as debug\n\nfn main():\n    pass\n";
    let section = |name: &str, flag: &str| format!("[cpp.{name}]\ninclude_paths = [\"{third_party}\"]\nflags = [\"{flag}\"]\n");
    package(&runtimes, &format!("[package]\nname = \"runtimes\"\nversion = \"0.1.0\"\n\n{}\n{}", section("release", release), section("debug", debug)), &[("main", program)]);
    std::fs::copy(entry.join("nlohmann_json.hpp"), runtimes.join("src").join("nlohmann_json.hpp")).unwrap();
    let checked = ember(&["check"], &runtimes);
    assert!(!checked.ok && checked.stderr.contains("error[E9020]") && checked.stderr.contains(&format!("`{release}`")) && checked.stderr.contains(&format!("`{debug}`")), "{}", checked.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `[BLD-6]`, `[PRF-3]` — `lto = "off" | "thin" | "on"`: each profile's own (`debug` off,
/// `development` thin, `release` on) unless the manifest's `[profiles.<name>] lto` says otherwise.
/// A value the C compiler does not do is replaced by the nearest one it does (of two as near, the
/// one that optimises more), and the build record, `inspect/<name>.build.json`, says what was asked,
/// what was done, and why when they differ. Any other word is `E9001`. No correctness property
/// depends on it: the program prints the same each time. With MSVC's `cl` the record is checked
/// against the object itself: link-time code generation's own format exactly when it says `on`.
#[test]
fn link_time_optimisation_is_the_profiles_or_the_manifests_and_recorded() {
    let dir = scratch();
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).expect("a C toolchain");
    let cl = matches!(&toolchain, ember_build::Toolchain::Msvc { cl, .. }
        if !cl.file_stem().is_some_and(|stem| stem.eq_ignore_ascii_case("clang-cl")));
    let program = "fn twice(x: int) -> int:\n    return x * 2\n\nfn main():\n    println(twice(21))\n";
    for (written, profile, asked) in [
        (None, "debug", "off"), (None, "development", "thin"), (None, "release", "on"),
        (Some("off"), "release", "off"), (Some("on"), "debug", "on"), (Some("thin"), "release", "thin"),
    ] {
        let manifest = match written {
            Some(lto) => format!("[package]\nname = \"lto\"\n\n[profiles.{profile}]\nlto = \"{lto}\"\n"),
            None => "[package]\nname = \"lto\"\n".to_string(),
        };
        package(&dir, &manifest, &[("main", program)]);
        // Compiled each time: a program the cache served would leave no object to look at.
        let output = ember_build::output_started(Command::new(EMBER).args(["run", "--profile", profile, "--out-dir", "out"])
            .env(ember_branding::no_compile_cache_var(), "1").current_dir(&dir)).expect("ember starts");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{profile}, lto {written:?}: {stderr}");
        assert_eq!(String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"), "42\n", "{profile}, lto {written:?}");
        let inspect = dir.join("out").join(profile).join("inspect");
        let record: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(inspect.join("main.build.json")).unwrap()).unwrap();
        let used = toolchain.lto(ember_build::Lto::from_name(asked).unwrap(), ember_build::Link::Here);
        assert_eq!(record["profile"], profile, "{record}");
        assert_eq!(record["lto"]["asked"], asked, "[BLD-6] {profile}, lto {written:?}: {record}");
        assert_eq!(record["lto"]["used"], used.name(), "[BLD-6] {profile}, lto {written:?}: {record}");
        assert_eq!(record["lto"]["why"].is_null(), used.name() == asked, "[BLD-6] the record says why only when they differ: {record}");
        if cl {
            // `/GL`'s objects begin with the anonymous header (0x0000, 0xFFFF); plain COFF with the machine.
            let object = std::fs::read(dir.join("out").join(profile).join("obj").join("main.obj")).unwrap();
            assert_eq!(object.starts_with(&[0, 0, 0xFF, 0xFF]), used == ember_build::Lto::On, "[BLD-6] {profile}, lto {written:?}");
        }
    }
    package(&dir, "[package]\nname = \"lto\"\n\n[profiles.release]\nlto = \"full\"\n", &[("main", program)]);
    let refused = ember(&["check"], &dir);
    assert!(!refused.ok && refused.stderr.contains("error[E9001]: `full` is not an `lto` value"), "{}", refused.stderr);
    assert!(refused.stderr.contains("\"off\", \"thin\" or \"on\""), "{}", refused.stderr);
    let _ = std::fs::remove_dir_all(&dir);
}

/// CMake, from the path or, on Windows, the copy Visual Studio's build tools carry; `None` without.
fn cmake() -> Option<PathBuf> {
    let works = |program: &Path| Command::new(program).arg("--version").output().is_ok_and(|output| output.status.success());
    if works(Path::new("cmake")) {
        return Some(PathBuf::from("cmake"));
    }
    let roots = ["C:\\Program Files\\Microsoft Visual Studio", "C:\\Program Files (x86)\\Microsoft Visual Studio"];
    let found = roots.iter().flat_map(|root| std::fs::read_dir(root).into_iter().flatten().flatten()).flat_map(|version| {
        std::fs::read_dir(version.path()).into_iter().flatten().flatten().map(|edition| {
            edition.path().join("Common7\\IDE\\CommonExtensions\\Microsoft\\CMake\\CMake\\bin\\cmake.exe")
        })
    });
    found.into_iter().find(|candidate| cfg!(windows) && works(candidate))
}

/// `[BLD-FFI-2]` — the toolchain's CMake module builds Ember packages as ordinary targets of a C
/// project: `ember_add_library(… KIND staticlib …)` is an imported library a C program links and
/// calls through the package's header, `KIND cdylib` a shared one, `ember_add_executable` an
/// imported program; `FLAGS_FROM app` makes `ember build --cc-flags-from-target app` take the C
/// target's compiler and flags from CMake's File API reply, so with MSVC the package is compiled
/// with the debug runtime a Debug `app` uses (`[BLD-FFI-1b]`). Without a reply the option says so.
#[test]
fn the_cmake_module_builds_packages_as_targets_of_a_c_project() {
    let missing = scratch();
    package(&missing, "[package]\nname = \"alone\"\nversion = \"0.1.0\"\n", &[("main", "fn main():\n    pass\n")]);
    let refused = ember(&["build", ".", "--cc-flags-from-target", "app"], &missing);
    assert!(!refused.ok && refused.stderr.contains("has no CMake File API reply"), "{}", refused.stderr);
    let _ = std::fs::remove_dir_all(&missing);
    let Some(cmake) = cmake() else { return };
    let dir = scratch();
    // The module `tools/generate_runtime.py` renders: `cmake/<Cli>.cmake`, its functions `<prefix>_add_…`.
    let cli = ember_branding::CLI_NAME;
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../cmake").join(format!("{}{}.cmake", cli[..1].to_uppercase(), &cli[1..]));
    let module = std::path::absolute(&module).unwrap().to_string_lossy().replace('\\', "/");
    let p = ember_branding::SYMBOL_PREFIX;
    let export = "@export(\"twice\")\npub fn twice(x: i32) -> i32:\n    return 2 * x\n";
    for (folder, kind, entry, text) in [("numbers", "staticlib", "lib", export), ("shared", "cdylib", "lib", export), ("tool", "bin", "main", "fn main():\n    println(\"tool\")\n")] {
        std::fs::create_dir_all(dir.join(folder).join("src")).unwrap();
        std::fs::write(dir.join(folder).join(ember_branding::MANIFEST), format!("[package]\nname = \"{folder}\"\nversion = \"0.1.0\"\nkind = \"{kind}\"\n")).unwrap();
        std::fs::write(dir.join(folder).join("src").join(ember_branding::source_file(entry)), text).unwrap();
    }
    std::fs::write(dir.join("main.c"), "#include \"numbers.h\"\n#include <stdio.h>\n\nint main(void) {\n    printf(\"%d\\n\", twice(21));\n    return 0;\n}\n").unwrap();
    std::fs::write(dir.join("shared.c"), "#include \"shared.h\"\n\nint uses_shared(void) { return twice(2); }\n").unwrap();
    let (lib, main) = (ember_branding::source_file("lib"), ember_branding::source_file("main"));
    std::fs::write(dir.join("CMakeLists.txt"), format!("\
cmake_minimum_required(VERSION 3.20)
project(demo C)
include(\"{module}\")
add_executable(app main.c)
{p}_add_library(numbers KIND staticlib PACKAGE numbers PROFILE development SOURCES numbers/src/{lib} FLAGS_FROM app)
target_link_libraries(app PRIVATE numbers)
{p}_add_library(shared KIND cdylib PACKAGE shared SOURCES shared/src/{lib})
add_library(user SHARED shared.c)
target_link_libraries(user PRIVATE shared)
{p}_add_executable(tool PACKAGE tool SOURCES tool/src/{main})
")).unwrap();
    let build = dir.join("build");
    let run = |args: &[&str]| {
        let output = Command::new(&cmake).args(args).current_dir(&dir).output().expect("cmake starts");
        (output.status.success(), format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)))
    };
    let (configured, log) = run(&["-S", ".", "-B", "build", &format!("-DEMBER_EXECUTABLE={EMBER}")]);
    assert!(configured, "{log}");
    let (built, log) = run(&["--build", "build", "--config", "Debug"]);
    assert!(built, "{log}");
    let files = walk(&build);
    let find = |name: &str| files.iter().find(|path| path.file_name().is_some_and(|file| file == name)).cloned();
    let app = find(if cfg!(windows) { "app.exe" } else { "app" }).expect("the C program");
    let output = Command::new(&app).output().expect("the C program runs");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");
    assert!(find(if cfg!(windows) { "main.exe" } else { "main" }).is_some(), "the Ember program was not built");
    // MSVC: the package took the Debug program's runtime switch, `/MDd`.
    if let Some(archive) = find("numbers.lib") {
        let bytes = std::fs::read(archive).unwrap();
        assert!(bytes.windows(7).any(|window| window == b"MSVCRTD"), "the package was not compiled with the program's `/MDd`");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Every file under `dir`.
fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else {
            out.push(path);
        }
    }
    out
}

/// `[BLD-FFI-4]` — Ember's emitted C and the project's C++ are compiled by one compiler: a
/// `[cpp.<project>]` section naming the compiler Ember's C is built with builds, and one naming
/// another is refused, naming both.
#[test]
fn one_compiler_builds_the_c_and_the_cpp() {
    let probe = ember(&["--version"], &std::env::temp_dir());
    let line = probe.stdout.lines().find(|line| line.starts_with("C compiler: ")).unwrap_or_default().to_string();
    let backend = match line.trim_start_matches("C compiler: ").split(' ').next().unwrap_or_default() {
        "msvc" if line.contains("clang-cl") => "clang-cl",
        name => name,
    }
    .to_string();
    let other = if backend == "gcc" { "msvc" } else { "gcc" };
    for (compiler, builds) in [(backend.as_str(), true), (other, false)] {
        let dir = scratch();
        package(
            &dir,
            &format!("[package]\nname = \"engine_user\"\nversion = \"0.1.0\"\n\n[cpp.engine]\ncompiler = \"{compiler}\"\nstandard = \"c++17\"\n"),
            &[("main", "import cpp \"engine.hpp\" with (project = \"engine\")\n\nfn main():\n    unsafe:  # SAFETY(ffi): a scalar call\n        println(cpp.twice(21))\n")],
        );
        std::fs::write(dir.join("src").join("engine.hpp"), "#pragma once\ninline int twice(int x) noexcept { return x * 2; }\n").unwrap();
        let built = ember(&["build"], &dir);
        if builds {
            assert!(built.ok, "{compiler}: {}", built.stderr);
        } else {
            assert!(!built.ok, "a C++ project compiled with {compiler} built with {backend}");
            assert!(
                built.stderr.contains(&format!("is compiled with `{compiler}`, and this build's C with `{backend}`: one compiler builds both ([BLD-FFI-4])")),
                "{}",
                built.stderr
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

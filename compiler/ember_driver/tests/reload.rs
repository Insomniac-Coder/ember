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

/// `[PRF-2]`, `[MAN-7]` — hot reload is a build mode of `debug` and `development`: a package asking
/// for it is checked in both. A `release` build ignores it and says so (ODR-118), whether the manifest
/// or `ember build --reload` asks, and builds the ordinary program.
#[test]
fn a_release_build_ignores_reload_with_a_note() {
    let root = workspace_root();
    let (_package, file) = package("reload-release", "[build]\nreload = \"all\"\n", "fn main():\n    pass\n");
    let file = file.to_string_lossy().into_owned();
    for profile in ["debug", "development"] {
        let checked = ember(&["check", "--profile", profile, &file], &root);
        assert_eq!(checked.exit, 0, "[PRF-2] reload was refused in `{profile}`:\n{}", checked.stderr);
        assert!(!checked.stderr.contains("hot reload is off"), "{profile}: {}", checked.stderr);
    }
    let checked = ember(&["check", "--profile", "release", &file], &root);
    assert_eq!(checked.exit, 0, "[MAN-7] a release build refused `reload`:\n{}", checked.stderr);
    assert!(checked.stderr.contains("note: hot reload is off in release builds"), "{}", checked.stderr);

    let (plain_package, plain) = package("reload-flag-release", "[package]\nname = \"p\"\n", "fn main():\n    pass\n");
    let out = plain_package.join("out");
    let built = ember(&["build", "--reload", "--profile", "release", &plain.to_string_lossy(), "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(built.exit, 0, "[MAN-7] `ember build --reload` failed in release:\n{}", built.stderr);
    assert!(built.stderr.contains("note: hot reload is off in release builds"), "{}", built.stderr);
    let bin = out.join("release").join("bin");
    let names: Vec<String> = std::fs::read_dir(&bin).map(|entries| entries.filter_map(|entry| entry.ok()).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect()).unwrap_or_default();
    // A reloadable build puts the shared runtime and the image, both shared libraries, beside the host.
    assert!(!names.is_empty() && names.iter().all(|name| !name.ends_with(".dll") && !name.ends_with(".so")), "a release build made a reloadable program: {names:?}");
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
/// Windows: no error box for a program this test starts. SEM_FAILCRITICALERRORS,
/// SEM_NOGPFAULTERRORBOX and SEM_NOOPENFILEERRORBOX, which every child inherits: a host that
/// crashes, or cannot start (a missing library, 0xc0000142), ends with its status instead of
/// opening a window that waits for a click (D-252).
#[cfg(windows)]
#[allow(unsafe_code)]
fn quiet_errors() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetErrorMode(mode: u32) -> u32;
    }
    // SAFETY: SetErrorMode only sets this process's error mode flags.
    unsafe {
        SetErrorMode(0x0001 | 0x0002 | 0x8000);
    }
}

#[cfg(not(windows))]
fn quiet_errors() {}

fn run_host(out: &Path) -> Run {
    quiet_errors();
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
    reload_run_with(label, manifest, &[], source, edits)
}

/// `reload_run`, with the files `files` written beside the source (C files an import compiles).
fn reload_run_with(label: &str, manifest: &str, files: &[(&str, &str)], source: &str, edits: &[&str]) -> (Run, Vec<Run>) {
    let root = workspace_root();
    let (package, file) = package(label, manifest, source);
    for (name, text) in files {
        std::fs::write(package.join(name), text).expect("the file is writable");
    }
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

/// `[HR-6]`, `[HR-6a]` — a function's address is its permanent thunk's: a function value made
/// before the reload and held across it calls the new body, as one the new image makes does.
/// `[HR-4a]`, `[HR-17]` — a static lives in the runtime's storage and keeps its value across the
/// reload.
#[test]
fn function_values_and_statics_outlive_their_image() {
    let v1 = "import std.hot\nfrom std.sync import Atomic\n\nstatic TICKS: Atomic[int] = Atomic(0)\n\n\
              fn step(i: int) -> int:\n    TICKS.fetch_add(1)\n    return i + 1\n\n\
              fn make() -> fn(int) -> int:\n    return step\n\n\
              fn twice(f: fn(int) -> int, x: int) -> int:\n    return f(f(x))\n\n\
              fn main():\n    held = step\n    println(twice(held, 0))\n    hot.checkpoint()\n    println(twice(held, 0))\n    \
              println(twice(make(), 1))\n    println(TICKS.load())\n";
    let v2 = v1.replace("return i + 1", "return i + 1000");
    let (run, reloads) = reload_run("reload-values", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "2\n2000\n2001\n6\n", "{}", run.stderr);
}

/// `[HR-10]`, `[HR-18]`, `[HR-18a]` — a `@noreload` function is called directly; a reload that
/// changes its body is refused, naming it, and the refusal is never partial: the reloadable body
/// edited beside it keeps running as it was.
#[test]
fn a_noreload_body_change_is_refused_whole() {
    let v1 = "import std.hot\n\n@noreload\nfn fixed(i: int) -> int:\n    return i * 2\n\n\
              fn step(i: int) -> int:\n    return i + 1\n\n\
              fn main():\n    for i in 0..2:\n        println(fixed(i), step(i))\n        hot.checkpoint()\n";
    let v2 = v1.replace("return i * 2", "return i * 3").replace("return i + 1", "return i + 100");
    let (run, reloads) = reload_run("reload-noreload", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "0 1\n2 2\n", "{}", run.stderr);
    let named = format!("the body of `@noreload` function `{}` changed", ember_branding::mangled("fixed"));
    assert!(run.stderr.contains("refused") && run.stderr.contains(&named), "{}", run.stderr);
}

/// `[HR-15]` — a closure's captured state is followed (ODR-126). A `Mote`'s only reference is the
/// one a closure captured, held in a `Ticker`'s field (the `Mote` keeps the `Ticker`, so the two
/// outlive the frame that made them). An edit that grows `Mote` moves the instance, and the runtime
/// finds and rewrites the reference through the closure's descriptor, which says what its captured
/// state holds. `[HR-15a]` — held in a `Box[dyn Speak]`, whose value's type the image does not know,
/// the reference cannot be found: plan refuses, saying how many it could not follow, and the old
/// code runs on.
#[test]
fn a_closure_holding_an_instance_follows_it_when_it_moves() {
    let tied = PARTICLES
        .replace("@export(\"game_frame\")", "class Mote:\n    n: int = 0\n    keep: Option[Ticker] = None\n\n\
                  class Ticker:\n    f: fn() -> int\n\n\
                  fn tie():\n    m = Mote()\n    step = owned fn() -> int:\n        m.n = m.n + 1\n        return m.n\n    \
                  m.keep = Some(Ticker(step))\n\n@export(\"game_frame\")")
        .replace("            for k in 0..3:\n", "            tie()\n            for k in 0..3:\n");
    let grow = |source: &str| source.replace("    n: int = 0\n", "    n: int = 0\n    more: int = 8\n");
    let run = host_run("reload-closure-moves", &tied, &[&grow(&tied)], 3);
    assert_eq!(run.stdout, "3\napplied, 1 migrated\n6\n9\n", "{}", run.stderr);

    let boxed = PARTICLES
        .replace("@export(\"game_frame\")", "interface Speak:\n    fn speak(self) -> int\n\n\
                  class Mote implements Speak:\n    n: int = 0\n    keep: Option[Holder] = None\n\n    \
                  fn speak(self) -> int:\n        return self.n\n\n\
                  class Holder:\n    s: Box[dyn Speak]\n\n\
                  fn tie():\n    m = Mote()\n    keeper = m\n    boxed: Box[dyn Speak] = Box(m)\n    keeper.keep = Some(Holder(boxed))\n\n@export(\"game_frame\")")
        .replace("            for k in 0..3:\n", "            tie()\n            for k in 0..3:\n");
    let run = host_run("reload-dyn-cannot-move", &boxed, &[&grow(&boxed)], 2);
    assert!(run.stdout.contains("an instance of `Mote` must move (its new size does not fit), but 1 of its 1 references are held where the runtime cannot follow them"), "{}{}", run.stdout, run.stderr);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
}

/// `[CG-C-11]` — a reloadable build with a `@fastmath` function builds its relaxed unit into the
/// image, the function's thunk staying in the main unit beside the image's tables; an edit to the
/// relaxed body is reloaded like any other.
#[test]
fn a_relaxed_function_is_reloaded() {
    let v1 = "import std.hot\n\n@fastmath\nfn scaled(i: int) -> f64:\n    return i as f64 * 1.5\n\n\
              fn main():\n    for i in 0..3:\n        println(scaled(i))\n        hot.checkpoint()\n";
    let v2 = v1.replace("* 1.5", "* 10.0");
    let (run, reloads) = reload_run("reload-relaxed", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "0.0\n10.0\n20.0\n", "{}", run.stderr);
}

/// `[HR-3b]` (A1, ODR-133) — a layout change applies at `std.hot.checkpoint()` when the old code
/// still running below it no longer depends on that layout: `Particle` loses a field, its instances
/// migrate in place, and `main`'s loop, which only hands the array to `tick` (reached through its
/// thunk, so the new code) and prints numbers, runs on with them. When `main` itself reads a
/// `Particle`'s field after the checkpoint returns, its old code would read the new layout with the
/// old offsets: the change is refused, naming `main` and the line of the call it is in.
#[test]
fn a_layout_change_applies_at_a_checkpoint_the_old_frames_do_not_depend_on() {
    let v1 = "import std.hot\n\nclass Particle:\n    x: int = 0\n    spare: int = 7\n    step: int = 1\n\n    \
              fn bump(mut self) -> int:\n        self.x = self.x + self.step\n        return self.x\n\n\
              fn tick(ps: Array[Particle]) -> int:\n    total = 0\n    for p in ps:\n        q = p\n        total += q.bump()\n    return total\n\n\
              fn main():\n    ps = [Particle(), Particle(), Particle()]\n    for i in 0..3:\n        println(tick(ps))\n        hot.checkpoint()\n";
    let v2 = v1.replace("    spare: int = 7\n", "").replace("self.x = self.x + self.step", "self.x = self.x + self.step * 10");
    let (run, reloads) = reload_run("reload-checkpoint-layout", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "3\n33\n63\n", "{}", run.stderr);
    assert!(!run.stderr.contains("refused"), "{}", run.stderr);

    let reads = v1.replace("        hot.checkpoint()\n", "        hot.checkpoint()\n        println(ps[0].step)\n");
    let reads_v2 = reads.replace("    spare: int = 7\n", "");
    let (run, reloads) = reload_run("reload-checkpoint-layout-used", RELOAD_ALL, &reads, &[&reads_v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "3\n1\n6\n1\n9\n1\n", "the old code did not run on: {}", run.stderr);
    assert!(run.stderr.contains("the layout of `Particle` changed, and `main`, live below this checkpoint, still uses its old layout after the call on line 23 returns"),
        "{}", run.stderr);
}

/// `[HR-12]`, `[HR-14]` — a reloadable build registers every instance in the live-instance list, so
/// a reload that removes a class while instances of it live is refused, naming the class and how
/// many there are; the old code runs on with its instances.
#[test]
fn removing_a_class_with_live_instances_is_refused() {
    let v1 = "import std.hot\n\nclass Particle:\n    x: int\n\n    fn init(self, x: int):\n        self.x = x\n\n\
              fn tick(n: int) -> int:\n    return n + 1\n\n\
              fn main():\n    ps = [Particle(1), Particle(2), Particle(3)]\n    println(tick(ps.len()))\n    hot.checkpoint()\n    \
              println(tick(ps[2].x))\n";
    let v2 = "import std.hot\n\nfn tick(n: int) -> int:\n    return n + 100\n\nfn main():\n    println(tick(0))\n";
    let (run, reloads) = reload_run("reload-class-removed", RELOAD_ALL, v1, &[v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "4\n4\n", "{}", run.stderr);
    assert!(run.stderr.contains("class `Particle` was removed while 3 instances of it live"), "{}", run.stderr);
}

/// `[HR-19]` — `reload = "bodies"`, the first tier, admits function-body changes and refuses a
/// change that needs instances migrated: a field added to a struct.
#[test]
fn the_bodies_tier_admits_bodies_and_refuses_layouts() {
    let manifest = "[build]\nreload = \"bodies\"\n";
    let v1 = "import std.hot\n\nstruct Point:\n    x: int\n\nfn show(p: Point) -> int:\n    return p.x\n\n\
              fn main():\n    p = Point(x = 1)\n    println(show(p))\n    hot.checkpoint()\n    println(show(p))\n    hot.checkpoint()\n    println(show(p))\n";
    let body = v1.replace("return p.x", "return p.x + 10");
    let layout = body.replace("    x: int\n", "    x: int\n    y: int = 0\n");
    let (run, reloads) = reload_run("reload-bodies", manifest, v1, &[&body]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.stdout, "1\n11\n11\n", "a body edit under `bodies`: {}", run.stderr);
    let (run, _) = reload_run("reload-bodies-layout", manifest, v1, &[&layout]);
    assert_eq!(run.stdout, "1\n1\n1\n", "a layout edit under `bodies`: {}", run.stderr);
    assert!(run.stderr.contains("the layout of `Point` changed") && run.stderr.contains("`reload = \"bodies\"` admits only body changes"), "{}", run.stderr);
}

/// `[HR-17a]` — editing a static's initialiser is refused, naming the static; a
/// `@reinit_on_reload` static runs its new initialiser instead and its old value is gone.
#[test]
fn a_static_initialiser_edit_needs_reinit_on_reload() {
    let v1 = "import std.hot\n\nstatic LIMIT: int = 10\n\n@reinit_on_reload\nstatic LABEL: str = \"old\"\n\n\
              fn show() -> String:\n    return f\"{LIMIT} {LABEL}\"\n\n\
              fn main():\n    println(show())\n    hot.checkpoint()\n    println(show())\n";
    let refused = v1.replace("LIMIT: int = 10", "LIMIT: int = 20");
    let (run, _) = reload_run("reload-static-init", RELOAD_ALL, v1, &[&refused]);
    assert_eq!(run.stdout, "10 old\n10 old\n", "{}", run.stderr);
    assert!(run.stderr.contains("the initialiser of static `LIMIT` changed"), "{}", run.stderr);
    let reinit = v1.replace("\"old\"", "\"new\"");
    let (run, _) = reload_run("reload-static-reinit", RELOAD_ALL, v1, &[&reinit]);
    assert_eq!(run.stdout, "10 old\n10 new\n", "{}", run.stderr);
    // A static the compiler folds into the code, whose initialiser's text is unchanged but whose
    // value changed with the function it calls: the old value cannot be kept, so it is refused too.
    let called = v1.replace("static LIMIT: int = 10", "fn ten() -> int:\n    return 10\n\nstatic LIMIT: int = ten()");
    let (run, _) = reload_run("reload-static-folded", RELOAD_ALL, &called, &[&called.replace("return 10", "return 30")]);
    assert_eq!(run.stdout, "10 old\n10 old\n", "{}", run.stderr);
    assert!(run.stderr.contains("the initialiser of static `LIMIT` changed"), "{}", run.stderr);
}

/// `[HR-6a]` — method tables and drop glue stay ordinary code pointers: an object made before the
/// reload keeps its method table, whose entries are thunks, so its virtual method and its `drop`
/// run the new bodies.
#[test]
fn objects_made_before_a_reload_run_the_new_methods() {
    let v1 = "import std.hot\n\nopen class Shape:\n    virtual fn area(self) -> int:\n        return 0\n\n\
              class Square(Shape):\n    side: int = 3\n\n    \
              override fn area(self) -> int:\n        return self.side * self.side\n\n    \
              fn drop(mut self):\n        println(\"square dropped\")\n\n\
              fn measure(s: Shape) -> int:\n    return s.area()\n\n\
              fn main():\n    s = Square()\n    println(measure(s))\n    hot.checkpoint()\n    println(measure(s))\n";
    let v2 = v1.replace("self.side * self.side", "self.side * 100").replace("square dropped", "square dropped by v2");
    let (run, reloads) = reload_run("reload-methods", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "9\n300\nsquare dropped by v2\n", "{}", run.stderr);
}

/// `[HR-3]`, `[HR-3a]`, `[HR-42]` — a reload applies only when no registered thread is in Ember
/// code: while a `thread.spawn` thread runs, the checkpoint leaves the old code running; once it is
/// joined, the next checkpoint applies the edit.
#[test]
fn a_running_thread_holds_the_reload_back() {
    let v1 = "import std.hot\nimport std.thread\nfrom std.sync import Atomic\n\n\
              static STARTED: Atomic[int] = Atomic(0)\nstatic STOP: Atomic[int] = Atomic(0)\n\n\
              fn version() -> int:\n    return 1\n\n\
              fn spin() -> int:\n    STARTED.store(1)\n    turns = 0\n    while STOP.load() == 0:\n        turns += 1\n    return turns\n\n\
              fn main():\n    worker = thread.spawn(fn() => spin())\n    while STARTED.load() == 0:\n        pass\n    \
              hot.checkpoint()\n    println(version())\n    STOP.store(1)\n    turns = worker.join()\n    hot.checkpoint()\n    println(version())\n    println(turns >= 0)\n";
    let v2 = v1.replace("return 1\n", "return 2\n");
    let (run, reloads) = reload_run("reload-threads", RELOAD_ALL, v1, &[&v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "1\n2\ntrue\n", "{}", run.stderr);
}

/// `[HR-27]` — a `release` build is the same whether or not the source uses any reload attribute:
/// with each attribute in place of a comment, so no line moves, the C is byte for byte the same.
#[test]
fn release_is_the_same_with_reload_attributes() {
    let root = workspace_root();
    let plain = "# the module's reload default would go here\n\n# an attribute would go here\nfn tuned(x: int) -> int:\n    return x * 3\n\n\
                 # an attribute would go here\nfn fixed(x: int) -> int:\n    return x + 1\n\n\
                 # an attribute would go here\nstatic LIMIT: int = 7\n\nfn main():\n    println(tuned(fixed(LIMIT)))\n";
    let attributed = plain
        .replacen("# the module's reload default would go here", "#! module noreload", 1)
        .replacen("# an attribute would go here", "@reloadable", 1)
        .replacen("# an attribute would go here", "@noreload", 1)
        .replacen("# an attribute would go here", "@reinit_on_reload", 1);
    let (package, file) = package("reload-release-identical", "[package]\nname = \"p\"\n", plain);
    let file_arg = file.to_string_lossy().into_owned();
    let without = ember(&["build", &file_arg, "--emit", "c", "--profile", "release"], &root);
    assert_eq!(without.exit, 0, "{}", without.stderr);
    std::fs::write(&file, &attributed).expect("the source is writable");
    let with = ember(&["build", &file_arg, "--emit", "c", "--profile", "release"], &root);
    assert_eq!(with.exit, 0, "{}", with.stderr);
    drop(package);
    assert!(without.stdout.len() > 1000, "no C was emitted");
    assert!(without.stdout == with.stdout, "[HR-27] the reload attributes changed a release build's C");
}

/// `[HR-28]` — `ember inspect --safety` reports, beside each function's checks, whether a call
/// reaches it through its thunk.
#[test]
fn inspect_safety_reports_the_thunk_indirection() {
    let root = workspace_root();
    let source = "@noreload\nfn fixed(xs: Array[int], i: int) -> int:\n    return xs[i]\n\nfn tuned(x: int) -> int:\n    return x * 2\n\n\
                  fn main():\n    println(tuned(fixed([1, 2], 1)))\n";
    let (package, file) = package("reload-inspect", RELOAD_ALL, source);
    let out = package.join("target");
    let built = ember(&["build", &file.to_string_lossy(), "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    let table = out.join("debug").join("inspect").join("main.safety.json");
    let report = ember(&["inspect", "--safety", &table.to_string_lossy()], &root);
    assert_eq!(report.exit, 0, "{}", report.stderr);
    let line = |name: &str| {
        report.stdout.lines().find(|line| line.trim_start().starts_with(&format!("{name} "))).unwrap_or_default().to_string()
    };
    assert!(line("fixed").contains("Bounds") && line("fixed").contains("Call: direct"), "{}", report.stdout);
    assert!(line("tuned").contains("Call: through its thunk"), "{}", report.stdout);
    assert!(line("main").contains("Call: through its thunk"), "{}", report.stdout);
}

/// `[HR-12a]` — the object header is the whole process's: once a program runs with the 40-byte
/// header of `reload = "all"`, an image built for `reload = "bodies"`'s 24-byte one cannot join it
/// (`E9035`), and nothing is built.
#[test]
fn an_image_for_the_other_header_is_e9035() {
    let root = workspace_root();
    let (package, file) = package("reload-header", RELOAD_ALL, "fn main():\n    println(1)\n");
    let out = package.join("target");
    let file_arg = file.to_string_lossy().into_owned();
    let out_arg = out.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    std::fs::write(package.join(MANIFEST), "[build]\nreload = \"bodies\"\n").expect("manifest is writable");
    let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
    assert_ne!(reloaded.exit, 0, "[HR-12a] an image for another header was built");
    assert!(reloaded.stderr.contains("error[E9035]") && reloaded.stderr.contains("40-byte headers, and this image's would have 24"), "{}", reloaded.stderr);
    assert!(!bin(&out).join(format!("main.image.1.{}", if cfg!(windows) { "dll" } else { "so" })).exists());
}

/// A C host of a reloadable program's images, as a game engine would be: it loads the first
/// image, calls the export `game_frame` once a frame and polls for a reload between frames, where
/// no Ember frame is live, printing what each frame returns and what each poll did.
const FRAME_HOST: &str = r#"#include "ember_rt.h"
#include <stdio.h>
#include <stdlib.h>
#if defined(_WIN32)
#define EXTENSION "dll"
#else
#define EXTENSION "so"
#endif

int main(int argc, char** argv) {
    if (argc < 3) {
        return 100;
    }
    char image[1200];
    char announce[1200];
    snprintf(image, sizeof image, "%s/main.image.0.%s", argv[1], EXTENSION);
    snprintf(announce, sizeof announce, "%s/main.reload", argv[1]);
    RT_reload_config config = { sizeof config, image, announce, 1 };
    if (RT_reload_init(&config) != 0) {
        return 101;
    }
    RT_rt_config runtime = RT_rt_config_default();
    RT_rt_init(&runtime);
    long long (*frame)(long long) = (long long (*)(long long))RT_reload_export("game_frame");
    if (frame == NULL) {
        return 102;
    }
    int frames = atoi(argv[2]);
    for (long long turn = 0; turn < frames; ++turn) {
        printf("%lld\n", frame(turn));
        fflush(stdout);
        int result = RT_reload_poll();
        const RT_reload_timings* stats = RT_reload_stats();
        if (result == RT_RELOAD_APPLIED) {
            printf("applied, %llu migrated\n", (unsigned long long)stats->instances_migrated);
        } else if (result != RT_RELOAD_IDLE) {
            printf("refused:\n%s", stats->report);
        }
        fflush(stdout);
    }
    return 0;
}
"#;

/// Build `source` as a reloadable program's first image, then each of `edits` as the next one,
/// and run `FRAME_HOST` over `frames` frames.
fn host_run(label: &str, source: &str, edits: &[&str], frames: u32) -> Run {
    let root = workspace_root();
    let (package, file) = package(label, RELOAD_ALL, source);
    let out = package.join("target");
    let out_arg = out.to_string_lossy().into_owned();
    let file_arg = file.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "the first build failed:\n{}", built.stderr);
    for edit in edits {
        std::fs::write(&file, edit).expect("the edit is writable");
        let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
        assert_eq!(reloaded.exit, 0, "{label}: the reload build failed:\n{}{}", reloaded.stdout, reloaded.stderr);
    }
    let program = link_host(&package, &out, FRAME_HOST);
    quiet_errors();
    let mut command = Command::new(&program);
    command.arg(bin(&out)).arg(frames.to_string()).env("LD_LIBRARY_PATH", bin(&out));
    let output = ember_build::output_started(&mut command).expect("the host runs");
    drop(package);
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// Link the C host `host` (written with `RT_` for the runtime's prefix) against the shared runtime,
/// into the package's output directory.
fn link_host(package: &Path, out: &Path, host: &str) -> PathBuf {
    let root = workspace_root();
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).expect("a C toolchain");
    let shared = ember_build::reload::shared_runtime(
        &toolchain,
        &runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX)),
        &includes,
        ember_build::Profile::Debug,
        Some(true),
        &ember_build::cache_root(),
        &bin(out),
    )
    .expect("the shared runtime builds");
    let source = package.join("frame_host.c");
    std::fs::write(&source, host.replace("RT_RELOAD", &format!("{}_RELOAD", ember_branding::SYMBOL_PREFIX.to_uppercase())).replace("RT_", ember_branding::RUNTIME_PREFIX))
        .expect("the host is writable");
    let program = bin(out).join(if cfg!(windows) { "frame_host.exe" } else { "frame_host" });
    let objects = package.join("obj");
    std::fs::create_dir_all(&objects).expect("the object directory is creatable");
    ember_build::reload::link_host_source(&toolchain, &source, &includes, ember_build::Profile::Debug, &shared, &objects, &program)
        .expect("the host links");
    program
}

const PARTICLES: &str = "from std.sync import Atomic, Mutex\n\n@sync\nclass Particle:\n    x: Atomic[int] = Atomic(0)\n    step: int = 1\n\n\
    static WORLD: Mutex[Array[Particle]] = Mutex([])\n\n\
    @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with world = WORLD.lock():\n        \
    if world.len() == 0:\n            for k in 0..3:\n                world.push(Particle())\n        \
    for p in world:\n            p.x.fetch_add(p.step)\n            total += p.x.load()\n    return total\n\nfn main():\n    pass\n";

/// `[HR-14]`, `[HR-15]`, `[HR-2a]`, `[HR-13a]`, `[HR-21]`, `[HR-30]` — a host polls between frames.
/// A field added with a default grows `Particle`, so each live instance moves: plan finds every
/// reference to it (in the `Mutex[Array[Particle]]` static, through the schema the package emits
/// for that standard type) and accounts for its count; prepare builds each new instance with the
/// old fields carried by name and the new one at its default; commit rewrites the references. The
/// next frame runs the new code on the migrated instances, their state kept. The export the host
/// holds is the permanent thunk, so the same pointer calls the new body.
#[test]
fn a_field_added_with_a_default_migrates_live_instances() {
    let v2 = PARTICLES.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n").replace("total += p.x.load()", "total += p.x.load() + p.bonus");
    let run = host_run("reload-migrate-add", PARTICLES, &[&v2], 3);
    assert_eq!(run.exit, 0, "{}\n{}", run.stdout, run.stderr);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n306\n309\n", "{}", run.stderr);
}

/// `[HR-14]` — the rest of the table, edit by edit, each against the running `PARTICLES`: a field
/// renamed with `@renamed_from` carries its value; a field widened losslessly is converted; a field
/// removed is dropped in reclaim, running its `drop` code (`[HR-2a]`); a field added with neither a
/// default nor a `Default` type is refused, naming it and suggesting a default; a field whose type
/// changed otherwise is refused, naming `migrate_from`; a base class changed is refused. A refusal
/// is a report and leaves the old code running (`[HR-18]`, `[HR-2]`).
#[test]
fn the_migration_table_by_name() {
    // Renamed: `step` (now 2) becomes `stride`, whose own default would be 1.
    let v1 = PARTICLES.replace("step: int = 1", "step: int = 2");
    let renamed = v1.replace("    step: int = 2\n", "    @renamed_from(\"step\")\n    stride: int = 1\n").replace("p.step", "p.stride");
    let run = host_run("reload-renamed", &v1, &[&renamed], 3);
    assert_eq!(run.stdout, "6\napplied, 3 migrated\n12\n18\n", "{}", run.stderr);
    // Without `@renamed_from` it is a removal and an addition: the new field starts at its default.
    let replaced = v1.replace("    step: int = 2\n", "    stride: int = 1\n").replace("p.step", "p.stride");
    let run = host_run("reload-replaced", &v1, &[&replaced], 3);
    assert_eq!(run.stdout, "6\napplied, 3 migrated\n9\n12\n", "{}", run.stderr);

    // Widened: `i32` to `i64`, the value kept.
    let narrow = PARTICLES.replace("step: int = 1", "step: i32 = 5").replace("p.x.fetch_add(p.step)", "p.x.fetch_add(p.step as int)");
    let wide = narrow.replace("step: i32 = 5", "step: i64 = 9").replace("p.step as int", "p.step");
    let run = host_run("reload-widened", &narrow, &[&wide], 3);
    assert_eq!(run.stdout, "15\napplied, 3 migrated\n30\n45\n", "{}", run.stderr);

    // Removed: the field's value is dropped in reclaim.
    let tracked = PARTICLES.replace(
        "@sync\nclass Particle:\n",
        "@sync\nclass Tag:\n    id: int = 0\n\n    fn drop(self):\n        println(\"tag dropped\")\n\n@sync\nclass Particle:\n    tag: Tag = Tag()\n",
    );
    let untracked = tracked.replace("    tag: Tag = Tag()\n", "");
    let run = host_run("reload-removed", &tracked, &[&untracked], 2);
    assert_eq!(run.stdout, "3\ntag dropped\ntag dropped\ntag dropped\napplied, 3 migrated\n6\n", "{}", run.stderr);

    // Added with no default: refused, naming the field.
    let undefaulted = PARTICLES.replace("    step: int = 1\n", "    step: int = 1\n    mass: Tag2\n\n    fn init(self):\n        self.mass = Tag2(w = 1)\n").replace(
        "@sync\nclass Particle:\n",
        "struct Tag2:\n    w: int\n\n@sync\nclass Particle:\n",
    );
    let run = host_run("reload-undefaulted", PARTICLES, &[&undefaulted], 2);
    assert_eq!(run.stdout.lines().next(), Some("3"), "{}", run.stderr);
    assert!(run.stdout.contains("refused:") && run.stdout.contains("field `mass` added to `Particle` with no default; give it one (`mass: Tag2 = ...`)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);

    // Changed type with no `migrate_from`: refused, naming it.
    let retyped = PARTICLES.replace("step: int = 1", "step: str = \"one\"").replace("p.x.fetch_add(p.step)", "p.x.fetch_add(p.step.len())");
    let run = host_run("reload-retyped", PARTICLES, &[&retyped], 2);
    assert!(run.stdout.contains("field `step` of `Particle` changed type from `i64` to `str`; `Particle` can declare `migrate_from`"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-14]` — a field added with no default whose type is `Default` is initialised to
/// `Default.default()` in every live instance: the type's own implementation, written by hand or
/// derived, whatever value it gives (here not zero). New instances still take what `init` gives.
#[test]
fn a_field_added_whose_type_is_default_takes_its_default() {
    for (label, mass) in [
        ("reload-default-written", "struct Mass:\n    w: int\n\nextend Mass implements Default:\n    fn default() -> Mass:\n        return Mass(w = 7)\n\n"),
        ("reload-default-derived", "@derive(Default)\nstruct Mass:\n    w: int = 7\n\n"),
    ] {
        let v2 = PARTICLES
            .replace("@sync\nclass Particle:\n", &format!("{mass}@sync\nclass Particle:\n"))
            .replace("    step: int = 1\n", "    step: int = 1\n    mass: Mass\n\n    fn init(self):\n        self.mass = Mass(w = 1)\n")
            .replace("total += p.x.load()", "total += p.x.load() + p.mass.w");
        let run = host_run(label, PARTICLES, &[&v2], 3);
        assert_eq!(run.exit, 0, "{label}: {}\n{}", run.stdout, run.stderr);
        assert_eq!(run.stdout, "3\napplied, 3 migrated\n27\n30\n", "{label}: {}", run.stderr);
    }
}

/// `[HR-14]` — a field changed to a range type migrates only when every live value passes the
/// type's `checked` (`[RNG-3]`): an `i32` widened into `int in 1..=10` keeps its values; one whose
/// values fall outside `int in 2..=10` is refused in plan, naming the field, the type and how many
/// fail, and the old code runs on. A range type's value changed back to its representation is
/// carried (`[RNG-2]`); a range edited narrower is a change of the field's type, tested the same
/// way. A static of a number changed to a range type is tested and converted as a field is.
#[test]
fn a_field_changed_to_a_range_type_is_tested_by_checked() {
    let v1 = PARTICLES.replace("step: int = 1", "step: i32 = 1").replace("p.x.fetch_add(p.step)", "p.x.fetch_add(p.step as int)");
    let ranged_from = |range: &str, default: u32| {
        PARTICLES
            .replace("@sync\nclass Particle:\n", &format!("type Step = int in {range}\n\n@sync\nclass Particle:\n"))
            .replace("step: int = 1", &format!("step: Step = {default}"))
    };
    let ranged = |range: &str| ranged_from(range, 5);
    let run = host_run("reload-range-passes", &v1, &[&ranged("1..=10")], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);
    let run = host_run("reload-range-fails", &v1, &[&ranged("2..=10")], 2);
    assert!(
        run.stdout.contains("refused: field `step` of `Particle` became the range type `Step`, and 3 values of it fail `Step.checked`"),
        "{}",
        run.stdout
    );
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
    // Back to the representation: carried.
    let run = host_run("reload-range-back", &ranged("1..=10"), &[&PARTICLES.replace("step: int = 1", "step: int = 9")], 3);
    assert_eq!(run.stdout, "15\napplied, 3 migrated\n30\n45\n", "{}", run.stderr);
    // The range narrowed past the live values: refused.
    let run = host_run("reload-range-narrowed", &ranged("1..=10"), &[&ranged_from("6..=10", 6)], 2);
    assert!(run.stdout.contains("and 3 values of it fail `Step.checked`"), "{}", run.stdout);
    assert!(run.stdout.ends_with("30\n"), "the old code did not run on: {}", run.stdout);

    // A static: `LIMIT` was computed on first access as 7 (`seven()` prints, so it runs then, not
    // while compiling), which `int in 0..10` holds and `int in 0..5` does not; the new image's own
    // `seven()` would give 4, but the static keeps its value (`[HR-17]`).
    let statics = |ty: &str, range: &str| {
        let seven = if ty == "int" { "fn seven() -> int:\n    println(\"init\")\n    return 7\n" } else { "fn seven() -> Small:\n    println(\"init\")\n    return Small.clamped(7)\n" };
        format!(
            "type Small = int in {range}\n\n{seven}\nstatic LIMIT: {ty} = seven()\n\n@export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    \
             x: int = LIMIT\n    return x + turn\n\nfn main():\n    pass\n"
        )
    };
    let run = host_run("reload-range-static", &statics("int", "0..10"), &[&statics("Small", "0..10")], 2);
    assert_eq!(run.stdout, "init\n7\napplied, 0 migrated\n8\n", "{}", run.stderr);
    let run = host_run("reload-range-static-fails", &statics("int", "0..5"), &[&statics("Small", "0..5")], 2);
    assert!(run.stdout.contains("refused: static `LIMIT` became the range type `Small`, and its value fails `Small.checked`"), "{}", run.stdout);
    assert!(run.stdout.ends_with("8\n"), "the old code did not run on: {}", run.stdout);
    // Folded into the code, a constant static has no value to migrate: its new type applies.
    let folded = |ty: &str| format!("type Small = int in 0..10\n\nstatic LIMIT: {ty} = 7\n\n@export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    x: int = LIMIT\n    return x + turn\n\nfn main():\n    pass\n");
    let run = host_run("reload-range-folded", &folded("int"), &[&folded("Small")], 2);
    assert_eq!(run.stdout, "7\napplied, 0 migrated\n8\n", "{}", run.stderr);
}

/// `[HR-14]` — an enum variant's payload changed migrates by the field rows, applied to the payload:
/// each live value keeps its variant by name (a variant added before it rewrites its tag), a payload
/// field widened is converted, one added whose type is `Default` takes `default()`; one added whose
/// type is not is refused, naming it; a variant removed while values hold it is refused with their
/// count; a payload field removed is dropped in reclaim, running its `drop`. A struct inside a
/// payload (`Option[Point]`) migrates depth-first when it changes.
#[test]
fn an_enum_payload_migrates_by_the_field_rows() {
    let v1 = PARTICLES
        .replace(
            "@sync\nclass Particle:\n",
            "enum Shape:\n    Dot\n    Circle(r: i32)\n\n@sync\nclass Particle:\n    shape: Shape = Shape.Circle(2)\n",
        )
        .replace("total += p.x.load()", "total += p.x.load() + size(p.shape)")
        .replace(
            "fn main():",
            "fn size(s: Shape) -> int:\n    match s:\n        Shape.Dot:\n            return 0\n        Shape.Circle(r):\n            return r as int\n\nfn main():",
        );
    // Widened, a `Default` field added, a variant added before `Circle`.
    let grown = v1
        .replace(
            "enum Shape:\n    Dot\n    Circle(r: i32)\n",
            "@derive(Default)\nstruct Mark:\n    w: int = 7\n\nenum Shape:\n    Square(side: int)\n    Dot\n    Circle(r: int, mark: Mark)\n",
        )
        .replace("Shape.Circle(2)", "Shape.Circle(2, Mark())")
        .replace("        Shape.Circle(r):\n            return r as int\n", "        Shape.Square(side):\n            return side\n        Shape.Circle(r, mark):\n            return r * 10 + mark.w\n");
    let run = host_run("reload-payload-grown", &v1, &[&grown], 3);
    assert_eq!(run.stdout, "9\napplied, 3 migrated\n87\n90\n", "{}", run.stderr);
    // Renamed with `@renamed_from`: carried. Without it, a removal and an addition, which starts at
    // `i32`'s default.
    let renamed = |attribute: &str| {
        v1.replace("Circle(r: i32)", &format!("Circle({attribute}radius: i32)"))
            .replace("        Shape.Circle(r):\n            return r as int\n", "        Shape.Circle(radius):\n            return radius as int\n")
    };
    let run = host_run("reload-payload-renamed", &v1, &[&renamed("@renamed_from(\"r\") ")], 3);
    assert_eq!(run.stdout, "9\napplied, 3 migrated\n12\n15\n", "{}", run.stderr);
    let run = host_run("reload-payload-replaced", &v1, &[&renamed("")], 3);
    assert_eq!(run.stdout, "9\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);
    // Added with a type that is not `Default`: refused.
    let undefaulted = v1
        .replace("enum Shape:\n    Dot\n    Circle(r: i32)\n", "struct Tag2:\n    w: int\n\nenum Shape:\n    Dot\n    Circle(r: i32, tag: Tag2)\n")
        .replace("Shape.Circle(2)", "Shape.Circle(2, Tag2(w = 1))")
        .replace("        Shape.Circle(r):\n", "        Shape.Circle(r, tag):\n");
    let run = host_run("reload-payload-undefaulted", &v1, &[&undefaulted], 2);
    assert!(run.stdout.contains("refused: field `tag` added to `Shape.Circle`, whose type `Tag2` is not `Default`"), "{}", run.stdout);
    assert!(run.stdout.ends_with("12\n"), "the old code did not run on: {}", run.stdout);
    // The variant the values hold removed: refused, with their count.
    let removed = v1
        .replace("enum Shape:\n    Dot\n    Circle(r: i32)\n", "enum Shape:\n    Dot\n    Ring(r: i32)\n")
        .replace("Shape.Circle(2)", "Shape.Ring(2)")
        .replace("        Shape.Circle(r):\n", "        Shape.Ring(r):\n");
    let run = host_run("reload-payload-removed", &v1, &[&removed], 2);
    assert!(run.stdout.contains("variant `Shape.Circle` was removed while 3 values of it live (field `shape` of `Particle`)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("12\n"), "the old code did not run on: {}", run.stdout);
    // A payload field removed: dropped in reclaim.
    let tagged = v1
        .replace(
            "enum Shape:\n    Dot\n    Circle(r: i32)\n",
            "@sync\nclass Tag:\n    id: int = 0\n\n    fn drop(self):\n        println(\"tag dropped\")\n\nenum Shape:\n    Dot\n    Circle(r: i32, tag: Tag)\n",
        )
        .replace("Shape.Circle(2)", "Shape.Circle(2, Tag())")
        .replace("        Shape.Circle(r):\n", "        Shape.Circle(r, tag):\n");
    let untagged = tagged
        .replace("    Circle(r: i32, tag: Tag)\n", "    Circle(r: i32)\n")
        .replace("Shape.Circle(2, Tag())", "Shape.Circle(2)")
        .replace("        Shape.Circle(r, tag):\n", "        Shape.Circle(r):\n");
    let run = host_run("reload-payload-dropped", &tagged, &[&untagged], 2);
    assert_eq!(run.stdout, "9\ntag dropped\ntag dropped\ntag dropped\napplied, 3 migrated\n12\n", "{}", run.stderr);

    // A struct inside an `Option`'s payload, grown: migrated depth-first.
    let spotted = PARTICLES
        .replace("@sync\nclass Particle:\n", "@derive(Copy)\nstruct Point:\n    y: int = 4\n\n@sync\nclass Particle:\n    spot: Option[Point] = Some(Point())\n")
        .replace("total += p.x.load()", "total += p.x.load() + p.spot.unwrap_or(Point()).y");
    let grown = spotted
        .replace("    y: int = 4\n", "    y: int = 4\n    z: int = 50\n")
        .replace("p.spot.unwrap_or(Point()).y", "p.spot.unwrap_or(Point()).y + p.spot.unwrap_or(Point()).z");
    let run = host_run("reload-payload-struct", &spotted, &[&grown], 3);
    assert_eq!(run.stdout, "15\napplied, 3 migrated\n168\n171\n", "{}", run.stderr);
}

/// The value-data program: `Point`s by value in an `Array` a static holds, each moved on by every
/// frame.
const POINTS: &str = "from std.sync import Mutex\n\nstruct Point:\n    x: int = 0\n    step: int = 1\n\n\
    static PTS: Mutex[Array[Point]] = Mutex([])\n\n\
    @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with pts = PTS.lock():\n        \
    if pts.len() == 0:\n            for k in 0..3:\n                pts.push(Point())\n        \
    for i in 0..pts.len():\n            pts[i].x += pts[i].step\n            total += pts[i].x\n    return total\n\nfn main():\n    pass\n";

/// `[HR-13]` — value-typed data is migrated through its container: `Point`s by value in an `Array`
/// a static holds take a field added with a default, each element rebuilt into a buffer of the new
/// size, their state kept; in an `Array` a class's instances hold, likewise; in a fixed array, in
/// place; in an `Array` of `Array`s, element by element; unit enums in an `Array` keep their
/// variants by name (`[HR-11a]`), and one whose variant was removed is refused, naming it; enums
/// with payloads in an `Array` migrate by the field rows (`[HR-14]`).
#[test]
fn value_data_migrates_through_its_container() {
    let grown = |source: &str| source.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n").replace("total += pts[i].x", "total += pts[i].x + pts[i].bonus");
    let run = host_run("reload-array-static", POINTS, &[&grown(POINTS)], 3);
    assert_eq!(run.stdout, "3\napplied, 0 migrated\n306\n309\n", "{}", run.stderr);

    // In a class's instances.
    let marked = PARTICLES
        .replace("@sync\nclass Particle:\n", "struct Mark:\n    w: int = 1\n\n@sync\nclass Particle:\n    marks: Array[Mark] = [Mark(), Mark()]\n")
        .replace("total += p.x.load()", "total += p.x.load() + p.marks[0].w + p.marks[1].w");
    let remarked = marked.replace("    w: int = 1\n", "    w: int = 1\n    z: int = 10\n").replace("p.marks[1].w", "p.marks[1].w + p.marks[1].z");
    let run = host_run("reload-array-instances", &marked, &[&remarked], 3);
    assert_eq!(run.stdout, "9\napplied, 3 migrated\n42\n45\n", "{}", run.stderr);

    // A fixed array, in place.
    let fixed = POINTS
        .replace("static PTS: Mutex[Array[Point]] = Mutex([])", "struct Pair:\n    two: [Point; 2]\n\nstatic PTS: Mutex[Pair] = Mutex(Pair(two = [Point(), Point()]))")
        .replace(
            "        if pts.len() == 0:\n            for k in 0..3:\n                pts.push(Point())\n        for i in 0..pts.len():\n            pts[i].x += pts[i].step\n            total += pts[i].x\n",
            "        for i in 0..2:\n            pts.two[i].x += pts.two[i].step\n            total += pts.two[i].x\n",
        );
    let refixed = fixed.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n").replace("total += pts.two[i].x", "total += pts.two[i].x + pts.two[i].bonus");
    let run = host_run("reload-array-fixed", &fixed, &[&refixed], 3);
    assert_eq!(run.stdout, "2\napplied, 0 migrated\n204\n206\n", "{}", run.stderr);

    // An `Array` of `Array`s.
    let rows = POINTS
        .replace("Mutex[Array[Point]]", "Mutex[Array[Array[Point]]]")
        .replace("                pts.push(Point())\n", "                pts.push([Point()])\n")
        .replace("            pts[i].x += pts[i].step\n            total += pts[i].x\n", "            pts[i][0].x += pts[i][0].step\n            total += pts[i][0].x\n");
    let rerows = rows.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n").replace("total += pts[i][0].x", "total += pts[i][0].x + pts[i][0].bonus");
    let run = host_run("reload-array-rows", &rows, &[&rerows], 3);
    assert_eq!(run.stdout, "3\napplied, 0 migrated\n306\n309\n", "{}", run.stderr);

    // Unit enums: reordered, kept by name; removed while held, refused.
    let modes = "from std.sync import Mutex\n\nenum Mode:\n    Slow\n    Fast\n\nstatic MODES: Mutex[Array[Mode]] = Mutex([])\n\n\
                 fn speed(mode: Mode) -> int:\n    match mode:\n        Mode.Slow:\n            return 1\n        Mode.Fast:\n            return 10\n\n\
                 @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with modes = MODES.lock():\n        \
                 if modes.len() == 0:\n            modes.push(Mode.Slow)\n            modes.push(Mode.Fast)\n        \
                 for m in modes:\n            total += speed(m)\n    return total + turn * 100\n\nfn main():\n    pass\n";
    let reordered = modes
        .replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Fast\n    Turbo\n    Slow\n")
        .replace("        Mode.Fast:\n            return 10\n", "        Mode.Fast:\n            return 10\n        Mode.Turbo:\n            return 50\n");
    let run = host_run("reload-array-modes", modes, &[&reordered], 2);
    assert_eq!(run.stdout, "11\napplied, 0 migrated\n111\n", "{}", run.stderr);
    let removed = modes.replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Slow\n").replace("            modes.push(Mode.Fast)\n", "").replace("        Mode.Fast:\n            return 10\n", "");
    let run = host_run("reload-array-modes-removed", modes, &[&removed], 2);
    assert!(run.stdout.contains("variant `Mode.Fast` was removed while static `MODES` holds it (an element of a container)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("111\n"), "the old code did not run on: {}", run.stdout);

    // Enums with payloads.
    let shapes = "from std.sync import Mutex\n\nenum Shape:\n    Dot\n    Circle(r: int)\n\nstatic SHAPES: Mutex[Array[Shape]] = Mutex([])\n\n\
                  fn size(s: Shape) -> int:\n    match s:\n        Shape.Dot:\n            return 0\n        Shape.Circle(r):\n            return r\n\n\
                  @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with shapes = SHAPES.lock():\n        \
                  if shapes.len() == 0:\n            shapes.push(Shape.Circle(4))\n            shapes.push(Shape.Dot)\n        \
                  for s in shapes:\n            total += size(s)\n    return total + turn * 100\n\nfn main():\n    pass\n";
    let reshaped = shapes
        .replace("    Circle(r: int)\n", "    Circle(r: int, scale: i64)\n    Square(side: int)\n")
        .replace("        Shape.Circle(r):\n            return r\n", "        Shape.Circle(r, scale):\n            return r * 10 + scale\n        Shape.Square(side):\n            return side\n")
        .replace("Shape.Circle(4)", "Shape.Circle(4, 1)");
    let run = host_run("reload-array-shapes", shapes, &[&reshaped], 2);
    assert_eq!(run.stdout, "4\napplied, 0 migrated\n140\n", "{}", run.stderr);

    // `SoA` columns: each element gathered from the old columns, migrated, and scattered into a
    // block of the new ones.
    let columns = POINTS
        .replace("struct Point:\n", "@derive(Copy)\nstruct Point:\n")
        .replace("static PTS: Mutex[Array[Point]] = Mutex([])", "static PTS: Mutex[SoA[Point]] = Mutex(SoA[Point].with_capacity(0))");
    let run = host_run("reload-soa-static", &columns, &[&grown(&columns)], 3);
    assert_eq!(run.stdout, "3\napplied, 0 migrated\n306\n309\n", "{}", run.stderr);
    // A column of a struct of the package's, migrated depth-first.
    let nested = columns
        .replace("@derive(Copy)\nstruct Point:\n", "@derive(Copy)\nstruct Inner:\n    v: int = 2\n\n@derive(Copy)\nstruct Point:\n    inner: Inner = Inner()\n")
        .replace("total += pts[i].x", "total += pts[i].x + pts[i].inner.v");
    let renested = nested.replace("    v: int = 2\n", "    v: int = 2\n    w: int = 50\n").replace("pts[i].inner.v", "pts[i].inner.v + pts[i].inner.w");
    let run = host_run("reload-soa-nested", &nested, &[&renested], 3);
    assert_eq!(run.stdout, "9\napplied, 0 migrated\n162\n165\n", "{}", run.stderr);

    // ECS storage (`[HR-13a]`): a component's `AosStorage` and an `SoA` component's `SoaStorage`,
    // standard-library classes whose schemas the package emits, migrate their values.
    let ecs = "import std.hot\nfrom std.ecs import World, Component, AosStorage, SoaStorage\n\n\
               @derive(Copy, Component)\nstruct Position:\n    x: int\n\n\
               @derive(Copy, Component)\n@component(layout=soa)\nstruct Velocity:\n    dx: int\n\n\
               fn setup(w: World):\n    for k in 0..3:\n        e = w.create()\n        w.add(e, Position(k))\n        w.add(e, Velocity(10))\n\n\
               fn report(w: World) -> int:\n    total = 0\n    match w.storage[Position]():\n        Some(any):\n            \
               ps = any as! AosStorage[Position]\n            for p in ps.dense:\n                total += p.x\n        None:\n            pass\n    \
               match w.storage[Velocity]():\n        Some(any):\n            vs = any as! SoaStorage[Velocity]\n            \
               for i in 0..vs.dense.len():\n                total += vs.dense[i].dx\n        None:\n            pass\n    return total\n\n\
               fn main():\n    w = World()\n    setup(w)\n    for i in 0..2:\n        println(report(w))\n        hot.checkpoint()\n";
    let grown_ecs = ecs
        .replace("struct Position:\n    x: int\n", "struct Position:\n    x: int\n    y: int = 100\n")
        .replace("struct Velocity:\n    dx: int\n", "struct Velocity:\n    dx: int\n    dy: int = 1000\n")
        .replace("total += p.x\n", "total += p.x + p.y\n")
        .replace("total += vs.dense[i].dx\n", "total += vs.dense[i].dx + vs.dense[i].dy\n");
    let (run, reloads) = reload_run("reload-ecs-storage", RELOAD_ALL, ecs, &[&grown_ecs]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "33\n3333\n", "{}", run.stderr);

    // Arena contents: an arena's collections are views (`[ARN-5]`), which no class object or
    // static may hold (`E3063`, `E7002`), so a host's poll never meets one; a frame live below a
    // checkpoint that holds one still depends on its elements' layout, and the change is refused
    // (`[HR-3b]`) rather than leave old values under a new layout.
    let arena = "import std.hot\nfrom std.collections import ArenaArray\n\nstruct Point:\n    x: int = 0\n    step: int = 1\n\n\
                 fn tick(xs: ArenaArray[Point]) -> int:\n    total = 0\n    for p in xs.iter():\n        total += p.x + p.step\n    return total\n\n\
                 fn main():\n    arena = Arena.with_capacity(4096)\n    xs: ArenaArray[Point] = ArenaArray[Point].with_capacity(arena, 4)\n    \
                 for k in 0..3:\n        _pushed = xs.push(Point())\n    for i in 0..2:\n        println(tick(xs))\n        hot.checkpoint()\n";
    let regrown = arena.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n").replace("total += p.x + p.step\n", "total += p.x + p.step + p.bonus\n");
    let (run, reloads) = reload_run("reload-arena-frame", RELOAD_ALL, arena, &[&regrown]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "3\n3\n", "the old code did not run on: {}", run.stderr);
    assert!(run.stderr.contains("changed, and `main`, live below this checkpoint, still uses its old layout"), "{}", run.stderr);
}

/// Build `source` with the C files `files` beside it as a reloadable program's first image, then
/// each of `edits` as the next one, and run `FRAME_HOST` over `frames` frames.
fn host_run_with(label: &str, files: &[(&str, &str)], source: &str, edits: &[&str], frames: u32) -> Run {
    let root = workspace_root();
    let (package, file) = package(label, RELOAD_ALL, source);
    for (name, text) in files {
        std::fs::write(package.join(name), text).expect("the file is writable");
    }
    let out = package.join("target");
    let out_arg = out.to_string_lossy().into_owned();
    let file_arg = file.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "{label}: the first build failed:\n{}", built.stderr);
    for edit in edits {
        std::fs::write(&file, edit).expect("the edit is writable");
        let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
        assert_eq!(reloaded.exit, 0, "{label}: the reload build failed:\n{}{}", reloaded.stdout, reloaded.stderr);
    }
    let program = link_host(&package, &out, FRAME_HOST);
    quiet_errors();
    let mut command = Command::new(&program);
    command.arg(bin(&out)).arg(frames.to_string()).env("LD_LIBRARY_PATH", bin(&out));
    let output = ember_build::output_started(&mut command).expect("the host runs");
    drop(package);
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        stderr: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
        exit: output.status.code().unwrap_or(-1),
    }
}

/// The C side of the foreign-object tests: widgets made by C, freed by `widget_free`, which says so.
const WIDGETS_H: &str = "#include <stdint.h>\ntypedef struct Widget Widget;\nWidget* widget_new(int32_t id);\n\
                         void widget_free(Widget* widget);\nint32_t widget_id(const Widget* widget);\n";
const WIDGETS_C: &str = "#include \"widgets.h\"\n#include <stdio.h>\n#include <stdlib.h>\nstruct Widget { int32_t id; };\n\
                         Widget* widget_new(int32_t id) { Widget* w = (Widget*)malloc(sizeof *w); w->id = id; return w; }\n\
                         void widget_free(Widget* w) { printf(\"widget %d freed\\n\", (int)w->id); fflush(stdout); free(w); }\n\
                         int32_t widget_id(const Widget* w) { return w->id; }\n";
const WIDGETS_OVERLAY: &str = "unsafe overlay c \"widgets.h\":\n    @ffi(destroys(widget_free))\n    type Widget\n\n    \
                               fn widget_new(id) -> returns_owned(destructor=widget_free) one\n";

/// `[HR-22]` — a foreign object Ember owns (a `ForeignBox`) is carried across a reload unchanged:
/// the instance holding it migrates (a field replaced beside it), the box's object and destructor
/// are the same afterwards, and nothing frees it then; only when an edit removes the field holding
/// it does reclaim drop the box, freeing the object once, by the destructor it was made with.
#[test]
fn a_foreign_object_ember_owns_is_carried_across() {
    let v1 = "import std.hot\nfrom std.ffi import ForeignBox\nimport c \"./widgets.c\" with (overlay = \"widgets_overlay.em\") as b\n\n\
              class Device:\n    w: ForeignBox[b.Widget]\n    n: int = 1\n    spare: int = 0\n\n    \
              fn init(self, owned w: ForeignBox[b.Widget]):\n        self.w = w\n\n\
              fn tick(d: Device) -> int:\n    unsafe:  # SAFETY(ffi): the box keeps the widget alive for the call\n        \
              return b.widget_id(d.w.as_ptr() as *b.Widget) as int + d.n\n\n\
              fn main():\n    d = Device(b.widget_new(7))\n    for i in 0..2:\n        println(tick(d))\n        hot.checkpoint()\n    \
              println(\"end\")\n";
    let files = [("widgets.h", WIDGETS_H), ("widgets.c", WIDGETS_C), ("widgets_overlay.em", WIDGETS_OVERLAY)];
    // `spare` gives way to `bonus`: the instance migrates in place, its box carried.
    let grown = v1.replace("    spare: int = 0\n", "    bonus: int = 100\n").replace("as int + d.n\n", "as int + d.n + d.bonus\n");
    let (run, reloads) = reload_run_with("reload-foreign-carried", RELOAD_ALL, &files, v1, &[&grown]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}\n{}", run.stdout, run.stderr);
    assert_eq!(run.stdout, "8\n108\nend\nwidget 7 freed\n", "{}", run.stderr);
    // The field holding it removed: reclaim drops the box, which frees the widget once, then.
    let removed = v1
        .replace("    w: ForeignBox[b.Widget]\n", "    other: int = 0\n")
        .replace("        self.w = w\n", "        pass\n")
        .replace("fn tick(d: Device) -> int:\n    unsafe:  # SAFETY(ffi): the box keeps the widget alive for the call\n        return b.widget_id(d.w.as_ptr() as *b.Widget) as int + d.n\n",
                 "fn tick(d: Device) -> int:\n    return d.n\n");
    let (run, reloads) = reload_run_with("reload-foreign-dropped", RELOAD_ALL, &files, v1, &[&removed]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.exit, 0, "{}\n{}", run.stdout, run.stderr);
    assert_eq!(run.stdout, "8\nwidget 7 freed\n1\nend\n", "{}", run.stderr);
}

/// The C++ side of the `migrate_from` tests: a function that throws on a negative argument, and one
/// declared `noexcept`, whose throw would end the process.
const SCALES_HPP: &str = "#pragma once\n#include <stdexcept>\n\
                          inline int checked_scale(int x) { if (x < 0) { throw std::runtime_error(\"negative\"); } return x * 10; }\n\
                          inline int fast_scale(int x) noexcept { return x * 10; }\n";

/// `PARTICLES`' next image, whose `Particle.migrate_from` builds `speed` with `call`, a C++ call.
fn scaled_particles(call: &str, attribute: &str) -> String {
    MIGRATED
        .replace("from std.hot import ReloadError\n", "from std.hot import ReloadError\nimport cpp \"scales.hpp\"\n")
        .replace("    fn migrate_from(old: ref OldSelf)", &format!("    {attribute}fn migrate_from(old: ref OldSelf)"))
        .replace(
            "return Ok(Particle(old.x.load() * 100, old.step * 10))",
            &format!("unsafe:  # SAFETY(ffi): a C++ call by value\n            return Ok(Particle(old.x.load(), {call} as int))"),
        )
}

/// `[HR-39]` — a foreign call inside `migrate_from` returns its failure as `ReloadError.Foreign`: a
/// C++ function that may throw returns its throw as a `CppError`, which `?` converts, so a throw
/// fails prepare (the reload refused, the old code running on) and never ends the process; one
/// that does not throw migrates. A call to a C++ function called `noexcept`, whose throw would end
/// the process, is rejected at compile time (`E2225`), there or through a function `migrate_from`
/// calls. `[HR-43]` — `@allow_reload_terminate` on the `migrate_from` admits it, and `ember tcb`
/// lists the use.
#[test]
fn a_foreign_failure_in_migrate_from_is_foreign() {
    let files = [("scales.hpp", SCALES_HPP)];
    // Throws for a negative step: refused, `Foreign`, the process alive.
    let negative = PARTICLES.replace("step: int = 1", "step: int = -1");
    let catching = scaled_particles("cpp.checked_scale(old.step as i32)?", "");
    let run = host_run_with("reload-foreign-throws", &files, &negative, &[&catching], 2);
    assert_eq!(run.exit, 0, "{}\n{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("refused: `Particle.migrate_from` returned Foreign(") && run.stdout.contains("negative"), "{}", run.stdout);
    assert!(run.stdout.ends_with("-6\n"), "the old code did not run on: {}", run.stdout);
    // Does not throw: migrated, `speed` from C++.
    let run = host_run_with("reload-foreign-scales", &files, PARTICLES, &[&catching], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n33\n63\n", "{}", run.stderr);
    // A C function's status other than its success (`[FFI-16]`): `Foreign` too.
    let with_status = |source: &str| {
        source.replace(
            "@sync\nclass Particle:\n",
            "@repr(i32)\n@non_exhaustive\n@ffi(status=Status, ok=Success, error=StatusError)\nenum Status:\n    Success = 0\n    Busy = 2\n\n\
             unsafe extern \"C\":\n    @ffi(result(status), link_name=\"check_c\")\n    safe fn check(n: i32) -> Status\n\n\
             pub extern \"C\" fn check_c(n: i32) -> i32:\n    return 2 if n > 0 else 0\n\n@sync\nclass Particle:\n",
        )
    };
    let status = with_status(MIGRATED)
        .replace("return Ok(Particle(old.x.load() * 100, old.step * 10))", "check(old.step as i32)?\n        return Ok(Particle(old.x.load(), 1))");
    let run = host_run("reload-foreign-status", &with_status(PARTICLES), &[&status], 2);
    assert!(run.stdout.contains("refused: `Particle.migrate_from` returned Foreign(Status.Busy)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);

    // `noexcept`: rejected while compiling, directly or through a function.
    let root = workspace_root();
    let direct = scaled_particles("cpp.fast_scale(old.step as i32)", "");
    let through = direct
        .replace("cpp.fast_scale(old.step as i32)", "scale(old.step)")
        .replace("fn main():", "fn scale(step: int) -> i32:\n    unsafe:  # SAFETY(ffi): a C++ call by value\n        return cpp.fast_scale(step as i32)\n\nfn main():");
    for (label, source, path) in [("reload-noexcept-direct", &direct, ""), ("reload-noexcept-through", &through, " through `scale`")] {
        let (dir, file) = package(label, RELOAD_ALL, PARTICLES);
        std::fs::write(dir.join("scales.hpp"), SCALES_HPP).expect("the header is writable");
        let out = dir.join("target").to_string_lossy().into_owned();
        let first = ember(&["build", &file.to_string_lossy(), "--out-dir", &out], &root);
        assert_eq!(first.exit, 0, "{label}: {}", first.stderr);
        std::fs::write(&file, source).expect("the edit is writable");
        let built = ember(&["build", "--reload", &file.to_string_lossy(), "--out-dir", &out], &root);
        drop(dir);
        assert_ne!(built.exit, 0, "{label}: a `noexcept` C++ call in `migrate_from` was built");
        assert!(built.stderr.contains("error[E2225]") && built.stderr.contains(&format!("`Particle.migrate_from` may not call `")) && built.stderr.contains(&format!("fast_scale`{path}, a C++ function called noexcept")),
            "{label}: {}", built.stderr);
    }
    // Admitted with `@allow_reload_terminate`, and listed by `ember tcb`.
    let admitted = scaled_particles("cpp.fast_scale(old.step as i32)", "@allow_reload_terminate\n    ");
    let run = host_run_with("reload-noexcept-admitted", &files, PARTICLES, &[&admitted], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n33\n63\n", "{}", run.stderr);
    // (`ember tcb` checks the source on its own, with no running program: `OldSelf` is its own.)
    let (dir, file) = package("reload-noexcept-tcb", RELOAD_ALL, &admitted.replace("old.step", "old.speed"));
    std::fs::write(dir.join("scales.hpp"), SCALES_HPP).expect("the header is writable");
    let report = ember(&["tcb", &file.to_string_lossy()], &root);
    drop(dir);
    assert_eq!(report.exit, 0, "{}", report.stderr);
    assert!(report.stdout.contains("Reload may end the process (1):") && report.stdout.contains("`Particle.migrate_from` is `@allow_reload_terminate`"), "{}", report.stdout);
    // Anywhere but a `migrate_from`, the attribute is `E0104`.
    let misplaced = PARTICLES.replace("fn main():", "@allow_reload_terminate\nfn helper() -> int:\n    return 1\n\nfn main():");
    let (package, file) = package("reload-terminate-misplaced", RELOAD_ALL, &misplaced);
    let checked = ember(&["check", &file.to_string_lossy()], &root);
    drop(package);
    assert!(checked.stderr.contains("error[E0104]") && checked.stderr.contains("`@allow_reload_terminate` applies to a type's `migrate_from`"), "{}", checked.stderr);
}

/// `[HR-20]` — an object a `Retained` token pins (foreign code may keep its address, `[FFI-23]`)
/// is never moved: a reload that would move it (its new size does not fit) is refused, naming the
/// class and how many are pinned, though every reference to it could be found and rewritten (the
/// token sits in an `Array` a static holds); an unpinned one moves as before. A change that fits
/// migrates a pinned object in place, its address kept.
#[test]
fn a_pinned_object_is_never_moved() {
    let v1 = "from std.sync import Mutex\nfrom std.ffi import Retained\n\n@sync\nclass Mote:\n    x: int = 1\n    spare: int = 0\n\n\
              static PINS: Mutex[Array[Retained[Mote]]] = Mutex([])\nstatic FREE: Mutex[Array[Mote]] = Mutex([])\n\n\
              @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with pins = PINS.lock():\n        \
              if pins.len() == 0:\n            pins.push(Retained.pin(Mote()))\n            pins.push(Retained.pin(Mote()))\n        \
              for tok in pins:\n            total += 1\n    \
              with free = FREE.lock():\n        if free.len() == 0:\n            free.push(Mote())\n        for m in free:\n            total += m.x * 10\n    \
              return total\n\nfn main():\n    pass\n";
    let grown = v1.replace("    spare: int = 0\n", "    spare: int = 0\n    more: int = 5\n");
    let run = host_run("reload-pinned-moves", v1, &[&grown], 2);
    assert_eq!(run.exit, 0, "{}\n{}", run.stdout, run.stderr);
    assert!(run.stdout.contains("refused: 2 instances of `Mote` must move (the new size does not fit), but a `Retained` token foreign code holds pins them"), "{}", run.stdout);
    assert!(run.stdout.ends_with("12\n"), "the old code did not run on: {}", run.stdout);
    // Without the pins, the same edit moves the instances.
    let unpinned = |source: &str| source.replace("            pins.push(Retained.pin(Mote()))\n            pins.push(Retained.pin(Mote()))\n", "            pass\n");
    let run = host_run("reload-unpinned-moves", &unpinned(v1), &[&unpinned(&grown)], 2);
    assert_eq!(run.stdout, "10\napplied, 1 migrated\n10\n", "{}", run.stderr);
    // A change that fits: migrated in place, pinned or not.
    let fits = v1.replace("    spare: int = 0\n", "    other: int = 3\n");
    let run = host_run("reload-pinned-in-place", v1, &[&fits], 2);
    assert_eq!(run.stdout, "12\napplied, 3 migrated\n12\n", "{}", run.stderr);
}

/// `[HR-11a]`, `[HR-14]` — an enum value migrates by variant name: reordering the variants
/// rewrites each value's discriminant; removing a variant a live value holds is refused, naming it.
#[test]
fn enum_values_migrate_by_variant_name() {
    let v1 = PARTICLES
        .replace("@sync\nclass Particle:\n", "enum Mode:\n    Slow\n    Fast\n\n@sync\nclass Particle:\n    mode: Mode = Mode.Fast\n")
        .replace("p.x.fetch_add(p.step)", "p.x.fetch_add(speed(p.mode))")
        .replace("fn main():", "fn speed(mode: Mode) -> int:\n    match mode:\n        Mode.Slow:\n            return 1\n        Mode.Fast:\n            return 10\n\nfn main():");
    let reordered = v1.replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Fast\n    Slow\n    Turbo\n")
        .replace("        Mode.Fast:\n            return 10\n", "        Mode.Fast:\n            return 10\n        Mode.Turbo:\n            return 100\n");
    let run = host_run("reload-enum-reordered", &v1, &[&reordered], 3);
    assert_eq!(run.stdout, "30\napplied, 3 migrated\n60\n90\n", "{}", run.stderr);
    let removed = v1.replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Slow\n").replace("mode: Mode = Mode.Fast", "mode: Mode = Mode.Slow")
        .replace("        Mode.Fast:\n            return 10\n", "");
    let run = host_run("reload-enum-removed", &v1, &[&removed], 2);
    assert!(run.stdout.contains("variant `Mode.Fast` was removed while 3 values of it live (field `mode` of `Particle`)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("60\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-11]` — every type in a reloadable package has a schema: its kind, base and layout facts, and
/// its fields in order (name, type schema hash, offset, size) or its variants (name, discriminant,
/// payload). A field's type schema hash is its type's own: `pair`'s is `Pair`'s.
#[test]
fn every_type_has_a_schema() {
    let root = workspace_root();
    let source = "open class Base:\n    id: int = 0\n\nstruct Pair:\n    a: u8 = 0\n    b: u32 = 0\n\n\
                  class Mote(Base):\n    x: f64 = 0.0\n    pair: Pair = Pair()\n\n\
                  enum Shape:\n    Dot\n    Circle(f64)\n\nenum Mode:\n    Slow\n    Fast\n\n\
                  fn main():\n    m = Mote()\n    _s = Shape.Circle(m.x)\n    _k = Mode.Fast\n    println(m.pair.b)\n";
    let (package, file) = package("reload-schema", RELOAD_ALL, source);
    let out = package.join("target");
    let built = ember(&["build", &file.to_string_lossy(), "--out-dir", &out.to_string_lossy()], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    let c = std::fs::read_to_string(out.join("debug").join("c").join("main.c")).expect("the generated C");
    // A type's row: name, kind, base, layout, size, align, schema hash, …
    let row = |name: &str| -> Vec<String> {
        let line = c.lines().find(|line| line.trim_start().starts_with(&format!("{{ \"{name}\", EMBER_RL_TYPE_")))
            .unwrap_or_else(|| panic!("[HR-11] no schema for `{name}`:\n{c}"));
        line.trim_start().trim_start_matches("{ ").split(", ").map(str::to_string).collect()
    };
    let (base, mote, pair, shape, mode) = (row("Base"), row("Mote"), row("Pair"), row("Shape"), row("Mode"));
    assert_eq!((base[1].as_str(), base[2].as_str()), ("EMBER_RL_TYPE_CLASS", "NULL"), "{base:?}");
    assert_eq!((mote[1].as_str(), mote[2].as_str()), ("EMBER_RL_TYPE_CLASS", "\"Base\""), "{mote:?}");
    assert_eq!(pair[1], "EMBER_RL_TYPE_STRUCT", "{pair:?}");
    assert_eq!((shape[1].as_str(), shape[3].as_str()), ("EMBER_RL_TYPE_ENUM", "0"), "{shape:?}");
    assert_eq!((mode[1].as_str(), mode[3].as_str()), ("EMBER_RL_TYPE_ENUM", "EMBER_RL_LAYOUT_UNIT"), "{mode:?}");
    for row in [&base, &mote, &pair, &shape, &mode] {
        assert!(row[4].contains("sizeof") && row[5].contains("_Alignof") && row[6].starts_with("UINT64_C(0x"), "{row:?}");
    }
    // A field: name, renamed from, type name, type schema hash, offset, size — in declaration order,
    // the base's first.
    let fields: Vec<Vec<String>> = c.lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("{ \"") && line.contains("_Mote*)0)->"))
        .map(|line| line.trim_start_matches("{ ").split(", ").map(str::to_string).collect())
        .collect();
    let names: Vec<&str> = fields.iter().map(|field| field[0].as_str()).collect();
    assert_eq!(names, ["\"id\"", "\"x\"", "\"pair\""], "{fields:?}");
    for field in &fields {
        // `offsetof(struct …, name)` is two of the pieces.
        assert!(field[4].contains("offsetof(") && field[6].contains("sizeof("), "{field:?}");
    }
    assert_eq!(fields[2][3], pair[6], "[HR-11] `pair`'s type schema hash is not `Pair`'s");
    // A variant: name, renamed from, discriminant, payload schema hash, payload fields.
    let variant = |name: &str| c.lines().map(str::trim_start).find(|line| line.starts_with(&format!("{{ \"{name}\", NULL, UINT64_C(")))
        .unwrap_or_else(|| panic!("[HR-11] no variant `{name}`:\n{c}")).to_string();
    assert!(variant("Dot").starts_with("{ \"Dot\", NULL, UINT64_C(0), ") && variant("Dot").ends_with("0u, NULL },"), "{}", variant("Dot"));
    assert!(variant("Circle").starts_with("{ \"Circle\", NULL, UINT64_C(1), ") && variant("Circle").contains("1u, "), "{}", variant("Circle"));
    assert!(variant("Fast").starts_with("{ \"Fast\", NULL, UINT64_C(1), "), "{}", variant("Fast"));
}

/// `[HR-17]`, `[HR-11a]` — a static of a unit enum whose variants were reordered keeps its variant,
/// its discriminant rewritten, alone or inside a struct; one holding a removed variant is refused.
#[test]
fn an_enum_static_keeps_its_variant() {
    let v1 = "from std.sync import Mutex\n\nenum Mode:\n    Slow\n    Fast\n\nstruct Setting:\n    mode: Mode = Mode.Slow\n\n\
              static START: Mode = Mode.Fast\nstatic NOW: Mutex[Setting] = Mutex(Setting())\n\n\
              fn speed(mode: Mode) -> int:\n    match mode:\n        Mode.Slow:\n            return 1\n        Mode.Fast:\n            return 10\n\n\
              @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    with now = NOW.lock():\n        \
              if turn == 0:\n            now.mode = Mode.Fast\n        return speed(START) * 100 + speed(now.mode)\n\nfn main():\n    pass\n";
    let reordered = v1
        .replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Fast\n    Turbo\n    Slow\n")
        .replace("        Mode.Fast:\n            return 10\n", "        Mode.Fast:\n            return 10\n        Mode.Turbo:\n            return 50\n");
    let run = host_run("reload-enum-static", v1, &[&reordered], 2);
    assert_eq!(run.stdout, "1010\napplied, 0 migrated\n1010\n", "{}", run.stderr);
    let removed = v1
        .replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Slow\n    Turbo\n")
        .replace("Mode.Fast", "Mode.Turbo");
    let run = host_run("reload-enum-static-removed", v1, &[&removed], 2);
    assert!(run.stdout.contains("variant `Mode.Fast` was removed while static `NOW` holds it (field `mode`)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("1010\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-15]`, `[HR-15a]`, `[HR-15b]` — an instance keeps its address where its new size fits: a
/// `Particle` that loses a field migrates in place. One that gains a field must move, and every
/// reference to it is rewritten: held in tuples, in `Box`es and in closures' captured state, which
/// the image describes to the runtime by its own layouts (ODR-126). Held as an interface value,
/// which the runtime cannot follow, a reference is not found, and plan refuses before prepare runs,
/// saying how many it could not follow. `[HR-13a]` — held in a `Map`, whose entries are payload
/// enums (`Option[MapSlot[int, Particle]]`) the package gives schemas, every reference is found and
/// rewritten, and the instances move.
#[test]
fn an_instance_moves_only_where_its_references_are_found() {
    let paired = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Array[(int, Particle)]] = Mutex([])")
        .replace("                world.push(Particle())", "                world.push((k, Particle()))")
        .replace("        for p in world:\n", "        for pair in world:\n            p = pair.1\n")
        .replace("    step: int = 1\n", "    step: int = 1\n    spare: int = 7\n");
    let smaller = paired.replace("    spare: int = 7\n", "");
    let run = host_run("reload-in-place", &paired, &[&smaller], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);
    let grow = |source: &str| source.replace("    step: int = 1\n", "    step: int = 1\n    more: int = 8\n");
    let larger = paired.replace("    spare: int = 7\n", "    spare: int = 7\n    more: int = 8\n");
    let run = host_run("reload-tuple-moves", &paired, &[&larger], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);

    let boxed = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Array[(int, Box[Particle])]] = Mutex([])")
        .replace("                world.push(Particle())", "                world.push((k, Box(Particle())))")
        .replace("        for p in world:\n            p.x.fetch_add(p.step)\n            total += p.x.load()",
                 "        for pair in world:\n            pair.1.x.fetch_add(pair.1.step)\n            total += pair.1.x.load()");
    let run = host_run("reload-box-moves", &boxed, &[&grow(&boxed)], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);

    let rows = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Array[Array[Particle]]] = Mutex([])")
        .replace("                world.push(Particle())", "                world.push([Particle()])")
        .replace("        for p in world:\n            p.x.fetch_add(p.step)\n            total += p.x.load()",
                 "        for row in world:\n            for p in row:\n                p.x.fetch_add(p.step)\n                total += p.x.load()");
    let run = host_run("reload-rows-move", &rows, &[&grow(&rows)], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);

    let mapped = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Map[int, Particle]] = Mutex({})")
        .replace("                world.push(Particle())", "                world[k] = Particle()")
        .replace("        for p in world:\n            p.x.fetch_add(p.step)\n            total += p.x.load()",
                 "        for k in 0..3:\n            p = world[k]\n            p.x.fetch_add(p.step)\n            total += p.x.load()");
    let grown = mapped.replace("    step: int = 1\n", "    step: int = 1\n    more: int = 8\n");
    let run = host_run("reload-map-moves", &mapped, &[&grown], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);
}

const MIGRATED: &str = "from std.sync import Atomic, Mutex\nfrom std.hot import ReloadError\n\n@sync\nclass Particle:\n    x: Atomic[int] = Atomic(0)\n    speed: int = 10\n\n    \
    fn init(self, x: int, speed: int):\n        self.x = Atomic(x)\n        self.speed = speed\n\n    \
    fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]:\n        return Ok(Particle(old.x.load() * 100, old.step * 10))\n\n\
    static WORLD: Mutex[Array[Particle]] = Mutex([])\n\n\
    @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    total: i64 = 0\n    with world = WORLD.lock():\n        \
    if world.len() == 0:\n            for k in 0..3:\n                world.push(Particle(0, 1))\n        \
    for p in world:\n            p.x.fetch_add(p.speed)\n            total += p.x.load()\n    return total\n\nfn main():\n    pass\n";

/// `[HR-16]`, `[HR-2a]`, `[HR-38]` — a type declaring `migrate_from(old: ref OldSelf)` is built by it:
/// `OldSelf` is the previous build's schema as `OldParticle`, which the runtime fills from each old
/// instance by field name, and the new instances replace the old ones, which reclaim drops field by
/// field: the old class's own `drop`, removed by this edit, is not called through its stub. The
/// generated struct is visible only as `OldSelf`: the package's own `OldParticle` is another type.
#[test]
fn migrate_from_builds_the_new_instances() {
    let own = |source: &str| source.replace("@sync\nclass Particle:\n", "struct OldParticle:\n    note: int = 5\n\n@sync\nclass Particle:\n");
    let dropping = own(PARTICLES).replace("    step: int = 1\n", "    step: int = 1\n\n    fn drop(self):\n        println(f\"dropped {self.x.load()}\")\n");
    let run = host_run("reload-migrate-from", &dropping, &[&own(MIGRATED)], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n330\n360\n", "{}", run.stderr);
}

/// `[HR-35]`, `[HR-34]`, `[HR-2]`, `[HR-37]` — a run-time check failing inside `migrate_from`
/// (an index out of bounds here) does not end the process: prepare fails with
/// `ReloadError.Migration`, naming the type, the instance and the check's message; nothing of the
/// new image is live and the old code runs on. `[HR-36]` — an allocation past the transaction
/// arena's bound fails it with `ReloadError.Allocation`.
#[test]
fn a_failing_migration_is_reported_not_fatal() {
    let bounds = MIGRATED.replace(
        "        return Ok(Particle(old.x.load() * 100, old.step * 10))",
        "        limits = [1, 2]\n        return Ok(Particle(limits[old.step + 5], 1))",
    );
    let run = host_run("reload-migrate-check", PARTICLES, &[&bounds], 2);
    assert!(run.stdout.contains("refused: Migration(Particle, 0, index 6 is out of bounds for a length of 2)"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
    let hungry = MIGRATED.replace(
        "        return Ok(Particle(old.x.load() * 100, old.step * 10))",
        "        big: Array[int] = []\n        big.reserve(100000000)\n        return Ok(Particle(big.len(), 1))",
    );
    let run = host_run("reload-migrate-alloc", PARTICLES, &[&hungry], 2);
    assert!(run.stdout.contains("refused: Allocation: the reload's transaction arena"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-17]` — a static whose struct type changed migrates by `[HR-14]`: a field added with a
/// default, the others kept.
#[test]
fn a_struct_static_migrates() {
    let v1 = "from std.sync import Mutex\n\nstruct Score:\n    points: int = 0\n\nstatic BOARD: Mutex[Score] = Mutex(Score())\n\n\
              @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    with board = BOARD.lock():\n        board.points += 5\n        return board.points\n\nfn main():\n    pass\n";
    let v2 = v1.replace("    points: int = 0\n", "    points: int = 0\n    bonus: int = 1000\n").replace("return board.points", "return board.points + board.bonus");
    let run = host_run("reload-static-struct", v1, &[&v2], 3);
    assert_eq!(run.stdout, "5\napplied, 0 migrated\n1010\n1015\n", "exit {}: {}", run.exit, run.stderr);
}

/// `ember run --hot` on `source`, with its standard streams piped: the child, and a channel of
/// each line it writes to standard output and to standard error, tagged.
fn start_hot(label: &str, source: &str) -> (test_directories::TemporaryDirectory, PathBuf, std::process::Child, std::sync::mpsc::Receiver<(bool, String)>) {
    use std::io::BufRead;
    quiet_errors();
    let root = workspace_root();
    let (package, file) = package(label, "[package]\nname = \"main\"\n", source);
    let out = package.join("target");
    let mut child = Command::new(EMBER)
        .args(["run", "--hot", &file.to_string_lossy(), "--out-dir", &out.to_string_lossy()])
        .current_dir(&root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("ember run --hot starts");
    let (sender, lines) = std::sync::mpsc::channel();
    for (stream, is_stdout) in [
        (Box::new(child.stdout.take().expect("piped")) as Box<dyn std::io::Read + Send>, true),
        (Box::new(child.stderr.take().expect("piped")), false),
    ] {
        let sender = sender.clone();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stream).lines().map_while(Result::ok) {
                if sender.send((is_stdout, line)).is_err() {
                    return;
                }
            }
        });
    }
    (package, file, child, lines)
}

/// Wait for a line containing `needle` on standard output (`stdout`) or error; everything seen,
/// for the failure's message.
fn wait_for(lines: &std::sync::mpsc::Receiver<(bool, String)>, stdout: bool, needle: &str, seen: &mut String) -> bool {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while let Some(left) = deadline.checked_duration_since(std::time::Instant::now()) {
        match lines.recv_timeout(left) {
            Ok((is_stdout, line)) => {
                seen.push_str(&format!("{}{line}\n", if is_stdout { "out: " } else { "err: " }));
                if is_stdout == stdout && line.contains(needle) {
                    return true;
                }
            }
            Err(_) => return false,
        }
    }
    false
}

/// `[HR-33]`, `[HR-31]` — `ember run --hot` builds under `debug`, runs the program and watches its
/// sources: an edit is built in the background while the program runs on, and applies at the
/// program's next `std.hot.checkpoint()`.
#[test]
fn run_hot_applies_an_edit_at_the_next_checkpoint() {
    // The program waits for the edit by time (two minutes, past `wait_for`'s minute), not by a count
    // of turns, which a fast machine finishes before the edit is built (D-760).
    let source = "import std.hot\nimport std.time\n\nfn version() -> int:\n    return 1\n\nfn main():\n    println(\"started\")\n    \
                  start = time.Instant.now()\n    while version() == 1 and start.elapsed().as_millis() < 120000:\n        hot.checkpoint()\n    \
                  println(f\"now {version()}\")\n";
    let (package, file, mut child, lines) = start_hot("reload-run-hot", source);
    let mut seen = String::new();
    let started = wait_for(&lines, true, "started", &mut seen);
    if started {
        std::fs::write(&file, source.replace("return 1\n", "return 2\n")).expect("the edit is writable");
    }
    let applied = started && wait_for(&lines, true, "now 2", &mut seen);
    if !applied {
        let _ = child.kill();
    }
    let status = child.wait().expect("ember run --hot ends");
    drop(package);
    assert!(applied, "the edit never ran:\n{seen}");
    assert!(status.success(), "{status:?}\n{seen}");
}

/// `[HR-33]` — a program `ember run --hot` runs that reaches no checkpoint is told so after ten
/// seconds, rather than appearing to ignore its edits.
#[test]
fn run_hot_tells_a_program_without_a_checkpoint() {
    use std::io::Write;
    let source = "fn main():\n    println(\"waiting\")\n    line = input()\n    println(f\"read {line}\")\n";
    let (package, _file, mut child, lines) = start_hot("reload-run-hot-quiet", source);
    let mut seen = String::new();
    let told = wait_for(&lines, true, "waiting", &mut seen) && wait_for(&lines, false, "has reached no `std.hot.checkpoint()` in ten seconds", &mut seen);
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"done\n");
    }
    let finished = wait_for(&lines, true, "read done", &mut seen);
    if !finished {
        let _ = child.kill();
    }
    let _ = child.wait();
    drop(package);
    assert!(told, "the program was not told:\n{seen}");
    assert!(finished, "the program did not finish:\n{seen}");
}

/// `[HR-21]` — an exported function's address is its permanent thunk, so the host's pointer stays
/// valid across every reload (the frame tests call the new body through it); and the exports are
/// the module protocol the host was built against: adding one, removing one or changing one's
/// signature is refused, since only a rebuilt host could call them.
#[test]
fn exports_are_the_module_protocol() {
    let added = PARTICLES.replace("fn main():", "@export(\"game_extra\")\nextern \"C\" fn extra() -> i64:\n    return 7\n\nfn main():");
    let run = host_run("reload-export-added", PARTICLES, &[&added], 2);
    assert!(run.stdout.contains("refused:") && run.stdout.contains("export `game_extra` was added, which changes the module protocol"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
    let changed = PARTICLES.replace("extern \"C\" fn frame(turn: i64) -> i64:", "extern \"C\" fn frame(turn: i32) -> i64:");
    let run = host_run("reload-export-changed", PARTICLES, &[&changed], 2);
    assert!(run.stdout.contains("the signature of export `game_frame` changed"), "{}", run.stdout);
}

/// `[HR-18b]` — `ember build --reload --explain` predicts the reload against the running program,
/// at its next checkpoint, and applies nothing: an edit to a `@noreload` body would be refused, and
/// says why; an edit to a reloadable body would apply. The program runs its old code throughout,
/// until `ember build --reload` announces an image.
#[test]
fn explain_predicts_against_the_running_program() {
    use std::io::BufRead;
    let root = workspace_root();
    // As above: the program waits for the announced image by time, not by a count of turns.
    let v1 = "import std.hot\nimport std.time\n\n@noreload\nfn fixed() -> int:\n    return 1\n\nfn version() -> int:\n    return 1\n\n\
              fn main():\n    println(\"started\")\n    start = time.Instant.now()\n    \
              while version() == 1 and start.elapsed().as_millis() < 120000:\n        \
              hot.checkpoint()\n    println(f\"now {version()} {fixed()}\")\n";
    let (package, file) = package("reload-explain", RELOAD_ALL, v1);
    let out = package.join("target");
    let (file_arg, out_arg) = (file.to_string_lossy().into_owned(), out.to_string_lossy().into_owned());
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "{}", built.stderr);
    quiet_errors();
    let mut program = Command::new(host(&out))
        .env(format!("{}_HOT", ember_branding::SYMBOL_PREFIX.to_uppercase()), "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("the program starts");
    let mut output = std::io::BufReader::new(program.stdout.take().expect("piped")).lines();
    assert_eq!(output.next().and_then(Result::ok).as_deref(), Some("started"));

    std::fs::write(&file, v1.replace("return 1\n\nfn version", "return 5\n\nfn version")).expect("the edit is writable");
    let refused = ember(&["build", "--reload", "--explain", &file_arg, "--out-dir", &out_arg], &root);
    std::fs::write(&file, v1.replace("return 1\n\n\nfn main", "return 2\n\n\nfn main").replace("fn version() -> int:\n    return 1", "fn version() -> int:\n    return 2"))
        .expect("the edit is writable");
    let applies = ember(&["build", "--reload", "--explain", &file_arg, "--out-dir", &out_arg], &root);
    let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
    let last = output.next().and_then(Result::ok);
    let finished = program.wait().expect("the program ends");
    drop(package);
    assert_ne!(refused.exit, 0, "{}{}", refused.stdout, refused.stderr);
    let named = format!("the body of `@noreload` function `{}` changed", ember_branding::mangled("fixed"));
    assert!(refused.stdout.contains("the reload would be refused") && refused.stdout.contains(&named), "{}{}", refused.stdout, refused.stderr);
    assert_eq!(applies.exit, 0, "{}{}", applies.stdout, applies.stderr);
    assert!(applies.stdout.contains("the reload would apply"), "{}{}", applies.stdout, applies.stderr);
    assert_eq!(reloaded.exit, 0, "{}", reloaded.stderr);
    assert_eq!(last.as_deref(), Some("now 2 1"), "the program did not run the announced image");
    assert!(finished.success());
}

/// `[HR-7]` — a function an edit removes keeps its thunk, which would reach a stub; while a value
/// of the function may be held (here by `main`'s own frame), or while old code that may call it
/// runs on (the frames live at a checkpoint), the thunk is reachable, so the reload is refused,
/// naming it.
#[test]
fn removing_a_function_whose_address_is_held_is_refused() {
    let v1 = "import std.hot\n\nfn helper(x: int) -> int:\n    return x + 1\n\nfn version() -> int:\n    return 1\n\n\
              fn main():\n    held = helper\n    println(held(1))\n    hot.checkpoint()\n    println(version())\n";
    let v2 = "import std.hot\n\nfn version() -> int:\n    return 2\n\nfn main():\n    hot.checkpoint()\n    println(version())\n";
    let (run, reloads) = reload_run("reload-removed-held", RELOAD_ALL, v1, &[v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.stdout, "2\n1\n", "{}", run.stderr);
    let named = format!("`{}` was removed, but its address may be held", ember_branding::mangled("helper"));
    assert!(run.stderr.contains(&named), "{}", run.stderr);

    // At a checkpoint the old `main` runs on, and calls it after the checkpoint.
    let v1 = "import std.hot\n\nfn helper(x: int) -> int:\n    return x + 1\n\nfn main():\n    hot.checkpoint()\n    println(helper(1))\n";
    let v2 = "import std.hot\n\nfn main():\n    hot.checkpoint()\n    println(0)\n";
    let (run, reloads) = reload_run("reload-removed-called", RELOAD_ALL, v1, &[v2]);
    assert_eq!(reloads[0].exit, 0, "{}", reloads[0].stderr);
    assert_eq!(run.stdout, "2\n", "{}", run.stderr);
    let named = format!("`{}` was removed while frames of the old code, which may call it, are live", ember_branding::mangled("helper"));
    assert!(run.stderr.contains(&named), "{}", run.stderr);
}

/// `[HR-13b]` — a value a raw pointer can reach may sit in foreign memory, where no reload can find
/// it: a change to its type's schema is refused, naming it.
#[test]
fn a_type_behind_a_raw_pointer_does_not_migrate() {
    let v1 = "from std.sync import Mutex\n\nstruct Score:\n    points: int = 0\n\nstatic BOARD: Mutex[Score] = Mutex(Score())\n\n\
              fn raw(p: *mut Score) -> int:\n    return 0\n\n\
              @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    with board = BOARD.lock():\n        board.points += 5\n        return board.points\n\nfn main():\n    pass\n";
    let v2 = v1.replace("    points: int = 0\n", "    points: int = 0\n    bonus: int = 1000\n");
    let run = host_run("reload-raw-pointer", v1, &[&v2], 2);
    assert!(run.stdout.contains("a value of `Score` may be reachable only through a raw pointer or foreign memory"), "{}", run.stdout);
    assert!(run.stdout.ends_with("10\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-14]`, `[LAY-2]` — a type's layout attributes are in its schema (`[HR-11]`): one changed on a
/// type a raw pointer can reach, as a type used across FFI is, is refused (the table's last row,
/// through `[HR-13b]`); on any other type the values migrate by name into the new layout.
#[test]
fn a_changed_layout_attribute_is_a_schema_change() {
    let v1 = "from std.sync import Mutex\n\nstruct Score:\n    level: u8 = 3\n    points: int = 0\n\nstatic BOARD: Mutex[Score] = Mutex(Score())\n\n\
              fn raw(p: *mut Score) -> int:\n    return 0\n\n\
              @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    with board = BOARD.lock():\n        board.points += 5\n        return board.points * 10 + board.level as int\n\nfn main():\n    pass\n";
    let aligned = v1.replace("struct Score:\n", "@align(64)\nstruct Score:\n");
    let run = host_run("reload-layout-foreign", v1, &[&aligned], 2);
    assert!(run.stdout.contains("a value of `Score` may be reachable only through a raw pointer or foreign memory"), "{}", run.stdout);
    assert!(run.stdout.ends_with("103\n"), "the old code did not run on: {}", run.stdout);
    let plain = v1.replace("fn raw(p: *mut Score) -> int:\n    return 0\n\n", "");
    let packed = plain.replace("struct Score:\n", "@packed\nstruct Score:\n");
    let run = host_run("reload-layout-packed", &plain, &[&packed], 2);
    assert_eq!(run.stdout, "53\napplied, 0 migrated\n103\n", "{}", run.stderr);
    // Crossing a foreign boundary by value, in a C-ABI function's signature: the foreign side was
    // compiled for its old layout attributes, so a change to them is refused.
    let crossing = plain.replace("fn main():", "extern \"C\" fn tally(s: Score) -> i64:\n    return s.points\n\nfn main():");
    let run = host_run("reload-layout-abi", &crossing, &[&crossing.replace("struct Score:\n", "@align(64)\nstruct Score:\n")], 2);
    assert!(run.stdout.contains("refused: the layout attributes of `Score` changed, and it crosses a foreign boundary by value"), "{}", run.stdout);
    assert!(run.stdout.ends_with("103\n"), "the old code did not run on: {}", run.stdout);
    // A field added changes what the foreign side passes as much: the C function's signature
    // changed, which only a rebuilt host can call (`[HR-21]`).
    let run = host_run("reload-layout-abi-field", &crossing, &[&crossing.replace("    points: int = 0\n", "    points: int = 0\n    bonus: int = 1\n")], 2);
    assert!(run.stdout.contains("refused: the signature of export `tally` changed"), "{}", run.stdout);
    assert!(run.stdout.ends_with("103\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-14]` — a class whose base changed is refused, and an enum variant renamed with
/// `@renamed_from` keeps the values that held it.
#[test]
fn a_changed_base_is_refused_and_a_renamed_variant_kept() {
    let based = PARTICLES.replace(
        "@sync\nclass Particle:\n",
        "@sync\nopen class Body:\n    mass: int = 1\n\n@sync\nopen class Shape:\n    sides: int = 4\n\n@sync\nclass Particle(Body):\n",
    );
    let rebased = based.replace("class Particle(Body):", "class Particle(Shape):");
    let run = host_run("reload-rebased", &based, &[&rebased], 2);
    assert!(run.stdout.contains("the base class of `Particle` changed"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);

    let v1 = PARTICLES
        .replace("@sync\nclass Particle:\n", "enum Mode:\n    Slow\n    Fast\n\n@sync\nclass Particle:\n    mode: Mode = Mode.Fast\n")
        .replace("p.x.fetch_add(p.step)", "p.x.fetch_add(speed(p.mode))")
        .replace("fn main():", "fn speed(mode: Mode) -> int:\n    match mode:\n        Mode.Slow:\n            return 1\n        Mode.Fast:\n            return 10\n\nfn main():");
    let renamed = v1
        .replace("enum Mode:\n    Slow\n    Fast\n", "enum Mode:\n    Slow\n    @renamed_from(\"Fast\")\n    Quick\n")
        .replace("Mode.Fast", "Mode.Quick");
    let run = host_run("reload-variant-renamed", &v1, &[&renamed], 3);
    assert_eq!(run.stdout, "30\napplied, 3 migrated\n60\n90\n", "{}", run.stderr);
}

/// `[HR-15]` — when an instance moves, its `Weak` handles are rewritten with its strong ones: a
/// `Weak` kept in another static upgrades to the moved instance.
#[test]
fn weak_handles_follow_a_moved_instance() {
    let v1 = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Array[Particle]] = Mutex([])\nstatic WATCH: Mutex[Array[Weak[Particle]]] = Mutex([])")
        .replace("                world.push(Particle())", "                p = Particle()\n                with watch = WATCH.lock():\n                    watch.push(Weak(p))\n                world.push(p)")
        .replace("    return total\n", "    with watch = WATCH.lock():\n        for w in watch:\n            match w.upgrade():\n                Some(p):\n                    total += 1000 * p.step\n                None:\n                    pass\n    return total\n");
    let v2 = v1.replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 100\n");
    let run = host_run("reload-weak", &v1, &[&v2], 3);
    assert_eq!(run.stdout, "3003\napplied, 3 migrated\n3006\n3009\n", "{}", run.stderr);
}

/// The benchmark's host: `FRAME_HOST` running until a frame returns `argv[2]` (or five minutes
/// pass), printing a frame's value only when it changes and each applied reload's phases.
const BENCH_HOST: &str = r#"#include "ember_rt.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#if defined(_WIN32)
#define EXTENSION "dll"
#else
#define EXTENSION "so"
#endif

int main(int argc, char** argv) {
    if (argc < 3) {
        return 100;
    }
    char image[1200];
    char announce[1200];
    snprintf(image, sizeof image, "%s/main.image.0.%s", argv[1], EXTENSION);
    snprintf(announce, sizeof announce, "%s/main.reload", argv[1]);
    RT_reload_config config = { sizeof config, image, announce, 1 };
    if (RT_reload_init(&config) != 0) {
        return 101;
    }
    RT_rt_config runtime = RT_rt_config_default();
    RT_rt_init(&runtime);
    long long (*frame)(long long) = (long long (*)(long long))RT_reload_export("game_frame");
    if (frame == NULL) {
        return 102;
    }
    long long stop = atoll(argv[2]);
    long long last = -1;
    time_t started = time(NULL);
    for (long long turn = 0; time(NULL) - started < 300; ++turn) {
        long long value = frame(turn);
        if (value != last) {
            printf("frame %lld\n", value);
            fflush(stdout);
            last = value;
            if (value == stop) {
                return 0;
            }
        }
        int result = RT_reload_poll();
        const RT_reload_timings* stats = RT_reload_stats();
        if (result == RT_RELOAD_APPLIED) {
            printf("applied: build %.1f ms, load %.1f ms, plan %.1f ms, prepare %.1f ms, commit %.1f ms, reclaim %.1f ms, %llu migrated\n",
                   stats->compile_ns / 1e6, stats->load_ns / 1e6, stats->plan_ns / 1e6, stats->prepare_ns / 1e6,
                   stats->commit_ns / 1e6, stats->reclaim_ns / 1e6, (unsigned long long)stats->instances_migrated);
        } else if (result == RT_RELOAD_REFUSED || result == RT_RELOAD_FAILED) {
            printf("refused:\n%s", stats->report);
        }
        fflush(stdout);
    }
    return 103;
}
"#;

/// The benchmark's program: 10,000 live `Particle`s, each moved on by every frame, and a frame
/// value saying which edit runs (`tag`) and that every instance is still there.
const BENCH_WORLD: &str = "from std.sync import Atomic, Mutex\n\n@sync\nclass Particle:\n    x: Atomic[int] = Atomic(0)\n    step: int = 1\n\n\
    static WORLD: Mutex[Array[Particle]] = Mutex([])\n\nfn tag(p: Particle) -> int:\n    return 1\n\n\
    @export(\"game_frame\")\nextern \"C\" fn frame(turn: i64) -> i64:\n    with world = WORLD.lock():\n        \
    if world.len() == 0:\n            for k in 0..10000:\n                world.push(Particle())\n        \
    for p in world:\n            p.x.fetch_add(p.step)\n        return tag(world[0]) * 100000 + world.len()\n\nfn main():\n    pass\n";

/// `[BEN-8]` — the end-to-end reload benchmark. A host runs a program holding 10,000 live
/// instances; three edits — a function body, a field added with a default, and a field whose type
/// changes through `migrate_from` — are each timed from the save (the source written) through
/// `ember build --reload` to the host's first frame running the new code. The figures, and the
/// runtime's split of each reload (`[HR-32]`), are printed with the configuration (`--nocapture`
/// shows them); rule HR-1's one-second budget is for `bench/bigpkg` on the reference machine, so it
/// is not asserted here.
#[test]
fn the_reload_benchmark_times_three_edits() {
    use std::io::BufRead;
    let body = BENCH_WORLD.replace("fn tag(p: Particle) -> int:\n    return 1\n", "fn tag(p: Particle) -> int:\n    return 2\n");
    let field = body
        .replace("    step: int = 1\n", "    step: int = 1\n    bonus: int = 3\n")
        .replace("fn tag(p: Particle) -> int:\n    return 2\n", "fn tag(p: Particle) -> int:\n    return p.bonus\n");
    let retyped = field
        .replace("from std.sync import Atomic, Mutex\n", "from std.sync import Atomic, Mutex\nfrom std.hot import ReloadError\n")
        .replace(
            "    step: int = 1\n    bonus: int = 3\n",
            "    step: f64 = 1.0\n    bonus: int = 3\n\n    fn init(self, x: int, step: f64, bonus: int):\n        self.x = Atomic(x)\n        \
             self.step = step\n        self.bonus = bonus\n\n    fn migrate_from(old: ref OldSelf) -> Result[Self, ReloadError]:\n        \
             return Ok(Particle(old.x.load(), old.step as f64, old.bonus + 1))\n",
        )
        .replace("world.push(Particle())", "world.push(Particle(0, 1.0, 3))")
        .replace("p.x.fetch_add(p.step)", "p.x.fetch_add(1)");
    let edits = [("a function body", body, 210000), ("a field added with a default", field, 310000), ("a field retyped through `migrate_from`", retyped, 410000)];

    let root = workspace_root();
    let (package, file) = package("reload-benchmark", RELOAD_ALL, BENCH_WORLD);
    let out = package.join("target");
    let out_arg = out.to_string_lossy().into_owned();
    let file_arg = file.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "the first build failed:\n{}", built.stderr);
    let program = link_host(&package, &out, BENCH_HOST);
    quiet_errors();
    let mut child = Command::new(&program)
        .arg(bin(&out))
        .arg("410000")
        .env("LD_LIBRARY_PATH", bin(&out))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("the host starts");
    let (sender, lines) = std::sync::mpsc::channel();
    let stdout = child.stdout.take().expect("piped");
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines().map_while(Result::ok) {
            if sender.send((true, line)).is_err() {
                return;
            }
        }
    });
    let mut seen = String::new();
    let mut figures = Vec::new();
    let mut running = wait_for(&lines, true, "frame 110000", &mut seen);
    for (what, source, value) in &edits {
        if !running {
            break;
        }
        let saved = std::time::Instant::now();
        std::fs::write(&file, source).expect("the edit is writable");
        let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
        assert_eq!(reloaded.exit, 0, "the reload build of {what} failed:\n{}", reloaded.stderr);
        let before = seen.len();
        running = wait_for(&lines, true, &format!("frame {value}"), &mut seen);
        let phases = seen[before..].lines().find(|line| line.contains("applied:")).unwrap_or("").trim_start_matches("out: ").to_string();
        figures.push((what, saved.elapsed(), phases));
    }
    if !running {
        let _ = child.kill();
    }
    let status = child.wait().expect("the host ends");
    drop(package);
    assert!(running, "an edit never ran:\n{seen}");
    assert!(status.success(), "{status:?}\n{seen}");
    assert!(!seen.contains("refused"), "{seen}");
    eprintln!("[BEN-8] save to the new code running, 10,000 live instances, debug profile, {}:", std::env::consts::OS);
    for (what, elapsed, phases) in &figures {
        eprintln!("  {what}: {:.0} ms ({phases})", elapsed.as_secs_f64() * 1000.0);
    }
    assert_eq!(figures.len(), 3);
    assert!(figures[1].2.ends_with("10000 migrated") && figures[2].2.ends_with("10000 migrated"), "{figures:?}");
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

/// `[FFI-39a]` — a trampoline calls the overrides through their permanent thunks (`[HR-6]`): an
/// Ember class deriving from a C++ class is made before a reload, C++ keeps calling its virtual
/// across it, and after the next checkpoint the call runs the override's new body.
#[test]
fn a_trampoline_reaches_an_edited_override_after_a_reload() {
    let root = workspace_root();
    let overlay = format!("engine_overlay.{}", ember_branding::SOURCE_EXT);
    let v1 = &(format!("import std.hot\nimport cpp \"engine.hpp\" with (overlay = \"{overlay}\")\n\n") + "\
              class Mine(cpp.Layer):\n    override fn on_update(self, step: i32) -> i32:\n        return step * 10\n\n\
              fn main():\n    layer = Mine()\n    unsafe:  # SAFETY(ffi): C++ calls the object's virtual while it lives\n        println(cpp.drive(layer))\n    \
              hot.checkpoint()\n    unsafe:  # SAFETY(ffi): as above\n        println(cpp.drive(layer))\n");
    let v2 = v1.replace("return step * 10", "return step * 1000");
    let (package, file) = package("reload-trampoline", RELOAD_ALL, v1);
    std::fs::write(
        package.join("engine.hpp"),
        "#pragma once\nclass Layer {\npublic:\n    Layer() noexcept {}\n    virtual ~Layer() {}\n    virtual int on_update(int step) noexcept { return step; }\n};\n\
         inline int drive(Layer& layer) noexcept { return layer.on_update(1) + layer.on_update(2); }\n",
    )
    .expect("the header is writable");
    std::fs::write(
        package.join(&overlay),
        "unsafe overlay cpp \"engine.hpp\":\n    @ffi(trampoline, virtuals=[\"on_update\"])\n    type Layer\n",
    )
    .expect("the overlay is writable");
    let out = package.join("target");
    let out_arg = out.to_string_lossy().into_owned();
    let file_arg = file.to_string_lossy().into_owned();
    let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(built.exit, 0, "the first build failed:\n{}", built.stderr);
    std::fs::write(&file, &v2).expect("the edit is writable");
    let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
    assert_eq!(reloaded.exit, 0, "the reload build failed:\n{}", reloaded.stderr);
    let run = run_host(&out);
    drop(package);
    assert_eq!(run.exit, 0, "{}", run.stderr);
    assert_eq!(run.stdout, "30\n3000\n", "{}", run.stderr);
}

/// `[HR-23]` — a changed C++ header refuses the reload in plan, naming the header (the
/// host was compiled against the old one), and the old image keeps running; a changed overlay
/// does not, since an overlay edit changes only Ember code, and the edited body beside it applies.
#[test]
fn a_changed_cpp_header_refuses_the_reload_and_an_overlay_does_not() {
    let root = workspace_root();
    let overlay = format!("engine_overlay.{}", ember_branding::SOURCE_EXT);
    let v1 = &(format!("import std.hot\nimport cpp \"engine.hpp\" with (overlay = \"{overlay}\")\n\n") + "\
              class Mine(cpp.Layer):\n    override fn on_update(self, step: i32) -> i32:\n        return step * 10\n\n\
              fn main():\n    layer = Mine()\n    unsafe:  # SAFETY(ffi): C++ calls the object's virtual while it lives\n        println(cpp.drive(layer))\n    \
              hot.checkpoint()\n    unsafe:  # SAFETY(ffi): as above\n        println(cpp.drive(layer))\n");
    let v2 = v1.replace("return step * 10", "return step * 1000");
    let header = "#pragma once\nclass Layer {\npublic:\n    Layer() noexcept {}\n    virtual ~Layer() {}\n    virtual int on_update(int step) noexcept { return step; }\n};\n\
                  inline int drive(Layer& layer) noexcept { return layer.on_update(1) + layer.on_update(2); }\n";
    let overlay_text = "unsafe overlay cpp \"engine.hpp\":\n    @ffi(trampoline, virtuals=[\"on_update\"])\n    type Layer\n";
    // Each case: the header and the overlay the reload is built with, and what the host prints.
    let changed_header = header.replace("inline int drive", "inline int spare() noexcept { return 0; }\ninline int drive");
    let changed_overlay = format!("# the engine's layer, which Ember classes derive from\n{overlay_text}");
    for (label, next_header, next_overlay, expected) in
        [("reload-header-changed", changed_header.as_str(), overlay_text, "30\n30\n"), ("reload-overlay-changed", header, changed_overlay.as_str(), "30\n3000\n")]
    {
        let (package, file) = package(label, RELOAD_ALL, v1);
        std::fs::write(package.join("engine.hpp"), header).expect("the header is writable");
        std::fs::write(package.join(&overlay), overlay_text).expect("the overlay is writable");
        let out = package.join("target");
        let out_arg = out.to_string_lossy().into_owned();
        let file_arg = file.to_string_lossy().into_owned();
        let built = ember(&["build", &file_arg, "--out-dir", &out_arg], &root);
        assert_eq!(built.exit, 0, "the first build failed:\n{}", built.stderr);
        std::fs::write(&file, &v2).expect("the edit is writable");
        std::fs::write(package.join("engine.hpp"), next_header).expect("the header is writable");
        std::fs::write(package.join(&overlay), next_overlay).expect("the overlay is writable");
        let reloaded = ember(&["build", "--reload", &file_arg, "--out-dir", &out_arg], &root);
        assert_eq!(reloaded.exit, 0, "the reload build failed:\n{}", reloaded.stderr);
        let run = run_host(&out);
        drop(package);
        assert_eq!(run.exit, 0, "{label}: {}", run.stderr);
        assert_eq!(run.stdout, expected, "{label}: {}", run.stderr);
        let refused = run.stderr.contains("refused: C++ header `engine.hpp` changed; the host must be rebuilt against it [HR-23]");
        assert_eq!(refused, label == "reload-header-changed", "{label}: {}", run.stderr);
    }
}

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
    assert!(run.stderr.contains("refused") && run.stderr.contains("the body of `@noreload` function `em_fixed` changed"), "{}", run.stderr);
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

/// `[HR-27]` — a `shipping` build is the same whether or not the source uses any reload attribute:
/// with each attribute in place of a comment, so no line moves, the C is byte for byte the same.
#[test]
fn shipping_is_the_same_with_reload_attributes() {
    let root = workspace_root();
    let plain = "# the module's reload default would go here\n\n# an attribute would go here\nfn tuned(x: int) -> int:\n    return x * 3\n\n\
                 # an attribute would go here\nfn fixed(x: int) -> int:\n    return x + 1\n\n\
                 # an attribute would go here\nstatic LIMIT: int = 7\n\nfn main():\n    println(tuned(fixed(LIMIT)))\n";
    let attributed = plain
        .replacen("# the module's reload default would go here", "#! module noreload", 1)
        .replacen("# an attribute would go here", "@reloadable", 1)
        .replacen("# an attribute would go here", "@noreload", 1)
        .replacen("# an attribute would go here", "@reinit_on_reload", 1);
    let (package, file) = package("reload-shipping-identical", "[package]\nname = \"p\"\n", plain);
    let file_arg = file.to_string_lossy().into_owned();
    let without = ember(&["build", &file_arg, "--emit", "c", "--profile", "shipping"], &root);
    assert_eq!(without.exit, 0, "{}", without.stderr);
    std::fs::write(&file, &attributed).expect("the source is writable");
    let with = ember(&["build", &file_arg, "--emit", "c", "--profile", "shipping"], &root);
    assert_eq!(with.exit, 0, "{}", with.stderr);
    drop(package);
    assert!(without.stdout.len() > 1000, "no C was emitted");
    assert!(without.stdout == with.stdout, "[HR-27] the reload attributes changed a shipping build's C");
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
        assert_eq!(reloaded.exit, 0, "the reload build failed:\n{}", reloaded.stderr);
    }
    let runtime = root.join("runtime").join(format!("{}_rt", ember_branding::SYMBOL_PREFIX));
    let includes = vec![runtime.join("include")];
    let requested = std::env::var(ember_branding::cc_var()).ok();
    let toolchain = ember_build::Toolchain::detect(requested.as_deref()).expect("a C toolchain");
    let shared = ember_build::reload::shared_runtime(
        &toolchain,
        &runtime.join("src").join(format!("{}rt.c", ember_branding::RUNTIME_PREFIX)),
        &includes,
        ember_build::Profile::Debug,
        true,
        &ember_build::cache_root(),
        &bin(&out),
    )
    .expect("the shared runtime builds");
    let source = package.join("frame_host.c");
    std::fs::write(&source, FRAME_HOST.replace("RT_RELOAD", &format!("{}_RELOAD", ember_branding::SYMBOL_PREFIX.to_uppercase())).replace("RT_", ember_branding::RUNTIME_PREFIX))
        .expect("the host is writable");
    let program = bin(&out).join(if cfg!(windows) { "frame_host.exe" } else { "frame_host" });
    let objects = package.join("obj");
    std::fs::create_dir_all(&objects).expect("the object directory is creatable");
    ember_build::reload::link_host_source(&toolchain, &source, &includes, ember_build::Profile::Debug, &shared, &objects, &program)
        .expect("the host links");
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
    assert!(run.stdout.contains("holds a variant of `Mode` that was removed"), "{}", run.stdout);
    assert!(run.stdout.ends_with("60\n"), "the old code did not run on: {}", run.stdout);
}

/// `[HR-15]`, `[HR-15a]`, `[HR-15b]` — an instance keeps its address where its new size fits: held
/// in a `Map`, whose slots the runtime cannot follow, a `Particle` that loses a field migrates in
/// place; one that gains a field must move, its references cannot all be found, and plan refuses
/// before prepare runs, saying how many it could not follow.
#[test]
fn an_instance_moves_only_where_its_references_are_found() {
    let mapped = PARTICLES
        .replace("static WORLD: Mutex[Array[Particle]] = Mutex([])", "static WORLD: Mutex[Map[int, Particle]] = Mutex({})")
        .replace("                world.push(Particle())", "                world[k] = Particle()")
        .replace("        for p in world:\n            p.x.fetch_add(p.step)\n            total += p.x.load()",
                 "        for k in 0..3:\n            p = world[k]\n            p.x.fetch_add(p.step)\n            total += p.x.load()")
        .replace("    step: int = 1\n", "    step: int = 1\n    spare: int = 7\n");
    let smaller = mapped.replace("    spare: int = 7\n", "");
    let run = host_run("reload-in-place", &mapped, &[&smaller], 3);
    assert_eq!(run.stdout, "3\napplied, 3 migrated\n6\n9\n", "{}", run.stderr);
    let larger = mapped.replace("    spare: int = 7\n", "    spare: int = 7\n    more: int = 8\n");
    let run = host_run("reload-cannot-move", &mapped, &[&larger], 2);
    assert!(run.stdout.contains("an instance of `Particle` must move (its new size does not fit), but 1 of its 1 references are held where the runtime cannot follow them"), "{}", run.stdout);
    assert!(run.stdout.ends_with("6\n"), "the old code did not run on: {}", run.stdout);
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
/// instance by field name, and the new instances replace the old ones, whose fields are dropped.
#[test]
fn migrate_from_builds_the_new_instances() {
    let run = host_run("reload-migrate-from", PARTICLES, &[MIGRATED], 3);
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
    let source = "import std.hot\n\nfn version() -> int:\n    return 1\n\nfn main():\n    println(\"started\")\n    turns = 0\n    \
                  while version() == 1 and turns < 20000000:\n        hot.checkpoint()\n        turns += 1\n    println(f\"now {version()}\")\n";
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

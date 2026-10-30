//! `[BLD-1]`, `[BLD-2]`, `[BLD-8]` — the module as the unit of front-end caching and the function as
//! the unit of checking within it: a package checked clean and checked again after each of a run
//! of edits reports exactly what a clean check of it reports, while checking again only the
//! functions an edit reaches. `[BLD-4]`: built through Ninja one C unit per module, as a clean build
//! writes it. `[BLD-7]`: on one thread or several, alike.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const EMBER: &str = env!("CARGO_BIN_EXE_ember");
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{}-incremental-{}-{}",
        ember_branding::CLI_NAME,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(path.join("src")).expect("scratch directory");
    path
}

/// A check's result: its exit status and what it reported (`--timings=json`'s line taken out); the
/// modules it checked, whole or in part, by file name (`None` when it kept none: a clean check);
/// and the functions it checked in the modules it checked in part, by symbol.
struct Checked {
    ok: bool,
    report: String,
    modules: Option<Vec<String>>,
    functions: Vec<String>,
}

fn check(dir: &Path, out: &str, clean: bool) -> Checked {
    let mut command = Command::new(EMBER);
    command.args(["check", "--out-dir", out, "--timings=json"]).current_dir(dir);
    if clean {
        command.env(ember_branding::no_incremental_var(), "1");
    } else {
        command.env_remove(ember_branding::no_incremental_var());
    }
    let output = ember_build::output_started(&mut command).expect("ember starts");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n");
    let mut report = String::new();
    let mut modules = None;
    let mut functions = Vec::new();
    let listed = |stage: &serde_json::Value| -> Vec<String> {
        stage["module"].as_str().unwrap_or_default().split(", ").filter(|name| !name.is_empty()).map(str::to_string).collect()
    };
    for line in stderr.lines() {
        if line.starts_with("{\"") && line.contains("\"timings\"") {
            let timings: serde_json::Value = serde_json::from_str(line).expect("timings are JSON");
            for stage in timings["timings"].as_array().expect("stages") {
                if stage["stage"] == "modules checked" {
                    modules = Some(listed(stage).iter().filter_map(|name| name.rsplit('/').next().map(str::to_string)).collect());
                }
                if stage["stage"] == "functions checked" {
                    functions = listed(stage);
                }
            }
        } else {
            report.push_str(line);
            report.push('\n');
        }
    }
    Checked { ok: output.status.success(), report, modules, functions }
}

/// A module's file, `geom.em`: its extension is the branding's (`[RT-5]`).
fn source(stem: &str) -> String {
    ember_branding::source_file(stem)
}

/// Each stem's file, in order.
fn sources(stems: &[&str]) -> Vec<String> {
    stems.iter().map(|stem| source(stem)).collect()
}

fn write(dir: &Path, file: &str, text: &str) {
    std::fs::write(dir.join("src").join(file), text).expect("source written");
}

/// Check the package incrementally and clean; the two must agree, and pass unless the step makes an
/// error (`ok`). The modules the incremental check checked, among the package's own (the standard
/// library's are kept after the first check), and the functions it checked in part of them.
fn agree(dir: &Path, step: &str, ok: bool) -> (Vec<String>, Vec<String>) {
    let incremental = check(dir, "target-incremental", false);
    let clean = check(dir, "target-clean", true);
    assert_eq!(incremental.ok, clean.ok, "{step}: exit status\nincremental:\n{}\nclean:\n{}", incremental.report, clean.report);
    assert_eq!(clean.ok, ok, "{step}: whether the package checks\n{}", clean.report);
    assert_eq!(incremental.report, clean.report, "{step}: what the checks report");
    assert!(clean.modules.is_none(), "{step}: a clean check keeps nothing");
    let mut modules: Vec<String> = incremental
        .modules
        .expect("an incremental check says what it checked")
        .into_iter()
        .filter(|name| sources(&["geom", "shapes", "extra", "main"]).contains(name))
        .collect();
    modules.sort();
    (modules, incremental.functions)
}

const GEOM: &str = r#"
pub struct Pt:
    pub x: int
    pub y: int

pub fn offset(x: int) -> int:
    return x + 1

pub fn larger[T: Ord](a: T, b: T) -> T:
    if a > b:
        return a
    return b

pub fn first(a: str, b: str) -> str:
    return a

pub fn pick(a: str, b: str) -> str:
    return first(a, b)

pub fn work(seed: int) -> int:
    acc = seed
    for i in 0..4:
        acc = (acc * 31 + i + 1) % 1009
    return acc

fn helper(p: Pt) -> int:
    unused = 3
    return larger(p.x, p.y)

pub fn score(p: Pt) -> int:
    return helper(p) + work(p.x)
"#;

const SHAPES: &str = r#"
import geom

pub open class Shape:
    pub size: int

    pub virtual fn area(self) -> int:
        return self.size

pub fn apply(xs: Array[int], f: fn(int) -> int) -> Array[int]:
    out: Array[int] = []
    for x in xs:
        out.push(f(x))
    return out

pub fn total(p: geom.Pt) -> int:
    k = 3
    ys = apply([p.x, p.y], fn(v) => v * k)
    s = Shape(geom.larger(ys[0], ys[1]))
    return s.area() + geom.score(p)

pub fn label(name: str) -> str:
    return geom.first(name, "shape")
"#;

const EXTRA: &str = r#"
pub fn leaf_value(x: int) -> int:
    return x * 2 + 1
"#;

const MAIN: &str = r#"
import geom
import shapes
import extra

fn main():
    p = geom.Pt(3, 4)
    println(shapes.total(p), extra.leaf_value(3), shapes.label("box"), geom.pick("a", "b"))
"#;

/// Each edit's check reports what a clean check reports, and checks again what the edit reaches:
/// `[BLD-8]` within a module, the function whose body changed and those of the module that use a
/// fact of it that changed; `[BLD-2]` beyond it, an importer's functions that use such a fact, every
/// function of a module that imports a changed declaration, and every module for a class.
#[test]
fn an_edited_package_checks_as_a_clean_one_does_and_checks_again_what_the_edit_reaches() {
    let dir = scratch();
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"shapes\"\nversion = \"0.1.0\"\n").expect("manifest");
    write(&dir, &source("geom"), GEOM);
    write(&dir, &source("shapes"), SHAPES);
    write(&dir, &source("extra"), EXTRA);
    write(&dir, &source("main"), MAIN);
    let geom_only = vec![source("geom")];
    let symbols = |names: &[&str]| -> Vec<String> { names.iter().map(|name| format!("{}{name}", ember_branding::mangle_prefix())).collect() };

    let all = sources(&["extra", "geom", "main", "shapes"]);
    assert_eq!(agree(&dir, "the first check", true).0, all, "the first check checks every module");
    assert_eq!(agree(&dir, "nothing edited", true), (vec![], vec![]), "nothing edited, nothing checked again");

    // A body: that function alone; its callers use no fact of it that changed.
    let geom = GEOM.replace("acc * 31 + i + 1", "acc * 37 + i + 2");
    write(&dir, &source("geom"), &geom);
    assert_eq!(agree(&dir, "a body", true), (geom_only.clone(), symbols(&["4geom4work"])));

    // A body above a generic its importer instantiates: the rest of the file, the generic and the
    // instance kept with the importer among it, moves; only the edited function is checked.
    let geom = geom.replace("return x + 1", "return x + 1000");
    write(&dir, &source("geom"), &geom);
    assert_eq!(agree(&dir, "a body above a generic", true), (geom_only.clone(), symbols(&["4geom6offset"])));

    // A body that changes what a caller's borrows read of it: its callers in its module and in its
    // importers are checked again, and nothing else.
    let geom = geom.replace("pub fn first(a: str, b: str) -> str:\n    return a", "pub fn first(a: str, b: str) -> str:\n    return b");
    write(&dir, &source("geom"), &geom);
    assert_eq!(
        agree(&dir, "a borrow fact", true),
        (vec![source("geom"), source("shapes")], symbols(&["4geom4pick", "4geom5first", "6shapes5label"]))
    );
    assert_eq!(agree(&dir, "nothing edited after it", true), (vec![], vec![]), "what was checked again is kept");

    // A signature in a leaf: the leaf and its importer, whole.
    write(&dir, &source("extra"), &EXTRA.replace("(x: int) -> int", "(x: i64) -> i64"));
    assert_eq!(agree(&dir, "a leaf's signature", true).0, sources(&["extra", "main"]));

    // A field of a widely imported type: every importer.
    write(&dir, &source("geom"), &geom.replace("    pub y: int\n", "    pub y: int\n    pub z: int = 0\n"));
    assert_eq!(agree(&dir, "a type", true).0, sources(&["geom", "main", "shapes"]));

    // A subclass overriding a virtual method: every module sees the class hierarchy.
    write(&dir, &source("main"), &format!("{MAIN}\nclass Square(shapes.Shape):\n    override fn area(self) -> int:\n        return self.size * self.size\n"));
    assert_eq!(agree(&dir, "a subclass", true).0, all);

    write(&dir, &source("main"), MAIN);
    assert_eq!(agree(&dir, "the subclass removed", true).0, all);

    // An error in one module: the kept modules' warnings are reported as a clean check reports them —
    // the body checks run on every function a type error is not in (`[IDE-3]`: `L1001` is a body
    // check's, in `geom`'s `helper`), and on all of them after an error a body check finds.
    write(&dir, &source("main"), &MAIN.replace("extra.leaf_value(3)", "extra.leaf_value(\"three\")"));
    assert_eq!(agree(&dir, "a type error", false).0, sources(&["main"]));
    let errored = check(&dir, "target-incremental", false);
    assert!(errored.report.contains("E2020") && errored.report.contains("L1001"), "{}", errored.report);
    write(&dir, &source("main"), &MAIN.replace("    p = geom.Pt(3, 4)\n", "    p = geom.Pt(3, 4)\n    xs: Array[int] = [1]\n    ys = xs\n    println(xs.len(), ys.len())\n"));
    assert_eq!(agree(&dir, "a body check's error", false).0, sources(&["main"]));
    let errored = check(&dir, "target-incremental", false);
    assert!(errored.report.contains("E3040") && errored.report.contains("L1001"), "{}", errored.report);

    // The error mended, `main` as it was: a check that found an error keeps nothing, so the record
    // `main` had before the errors holds, and nothing is checked again.
    write(&dir, &source("main"), MAIN);
    assert_eq!(agree(&dir, "the error mended", true), (vec![], vec![]));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A build: its status, what it reported, and each C unit it left (name and text), and when each
/// object was last written.
struct Built {
    ok: bool,
    report: String,
    units: std::collections::BTreeMap<String, Vec<u8>>,
    objects: std::collections::BTreeMap<String, std::time::SystemTime>,
}

/// `[CG-LL-1]` — the extension of the backend's units: LLVM IR, the default backend's, unless
/// `EMBER_BACKEND` names the C backend.
fn unit_extension() -> &'static str {
    if std::env::var("EMBER_BACKEND").is_ok_and(|backend| backend == "c") { ".c" } else { ".ll" }
}

fn build(dir: &Path, out: &str, clean: bool) -> Built {
    let mut command = Command::new(EMBER);
    command.args(["build", "--out-dir", out]).current_dir(dir).env(ember_branding::no_compile_cache_var(), "1");
    if clean {
        command.env(ember_branding::no_incremental_var(), "1");
    } else {
        command.env_remove(ember_branding::no_incremental_var());
    }
    let output = ember_build::output_started(&mut command).expect("ember starts");
    let report = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n").replace(out, "<out>");
    let listed = |folder: &str| -> Vec<(String, PathBuf)> {
        std::fs::read_dir(dir.join(out).join("debug").join(folder))
            .map(|entries| entries.flatten().map(|entry| (entry.file_name().to_string_lossy().into_owned(), entry.path())).collect())
            .unwrap_or_default()
    };
    let units = listed("c").into_iter().filter(|(name, _)| name.ends_with(unit_extension())).map(|(name, path)| (name, std::fs::read(path).expect("a unit reads"))).collect();
    let objects = listed("obj")
        .into_iter()
        .filter(|(name, _)| name.ends_with(".obj") || name.ends_with(".o"))
        .map(|(name, path)| (name, std::fs::metadata(path).and_then(|meta| meta.modified()).expect("an object's time")))
        .collect();
    Built { ok: output.status.success(), report, units, objects }
}

/// What the program built in `out` prints (the harness runs it).
fn runs(dir: &Path, out: &str) -> String {
    let exe = dir.join(out).join("debug").join("bin").join(if cfg!(windows) { "main.exe" } else { "main" });
    let output = ember_build::output_started(&mut Command::new(exe)).expect("the program starts");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

/// `[BLD-1]`, `[BLD-4]`, ODR-244, ODR-257 — `ember build` keeps the checked forms as `ember check`
/// does, writes a `debug` program one unit per module (C, or with the LLVM backend IR), and compiles
/// them through Ninja: after each edit, every unit is the one a clean build writes, byte for byte,
/// the program prints what the clean build's prints, and only the units whose text changed are
/// compiled again.
#[test]
fn an_edited_package_builds_as_a_clean_one_does_and_compiles_again_what_the_edit_changed() {
    let dir = scratch();
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"shapes\"\nversion = \"0.1.0\"\n").expect("manifest");
    write(&dir, &source("geom"), GEOM);
    write(&dir, &source("shapes"), SHAPES);
    write(&dir, &source("extra"), EXTRA);
    write(&dir, &source("main"), MAIN);
    let mut before = build(&dir, "target-incremental", false);
    assert!(before.ok, "{}", before.report);
    let units: Vec<&String> = before.units.keys().collect();
    for module in ["geom", "shapes", "extra"].map(|module| format!("main.m.{module}{}", unit_extension())) {
        assert!(units.iter().any(|unit| **unit == module), "one unit per module: {units:?}");
    }
    let edits = [
        ("a body", GEOM.replace("acc * 31 + i + 1", "acc * 37 + i + 2"), source("geom"), Some(vec!["main.m.geom"])),
        ("a body above a generic", GEOM.replace("acc * 31 + i + 1", "acc * 37 + i + 2").replace("return x + 1", "return x + 1000"), source("geom"), None),
        ("a leaf's signature", EXTRA.replace("(x: int) -> int", "(x: i64) -> i64"), source("extra"), None),
        ("a widely imported type", GEOM.replace("    pub y: int\n", "    pub y: int\n    pub z: int = 0\n"), source("geom"), None),
    ];
    for (step, text, file, recompiled) in edits {
        write(&dir, &file, &text);
        let incremental = build(&dir, "target-incremental", false);
        let clean = build(&dir, "target-clean", true);
        assert!(incremental.ok && clean.ok, "{step}:\n{}\n{}", incremental.report, clean.report);
        assert_eq!(incremental.report, clean.report, "{step}: what the builds report");
        assert_eq!(incremental.units.keys().collect::<Vec<_>>(), clean.units.keys().collect::<Vec<_>>(), "{step}: the C units");
        for (name, text) in &incremental.units {
            assert!(clean.units[name] == *text, "{step}: `{name}` is not the clean build's");
        }
        assert_eq!(runs(&dir, "target-incremental"), runs(&dir, "target-clean"), "{step}: what the programs print");
        if let Some(recompiled) = recompiled {
            let compiled: Vec<&str> = incremental
                .objects
                .iter()
                .filter(|(name, time)| before.objects.get(*name) != Some(*time))
                .map(|(name, _)| name.rsplit_once('.').map_or(name.as_str(), |(stem, _)| stem))
                .collect();
            assert_eq!(compiled, recompiled, "{step}: the objects compiled again");
        }
        before = incremental;
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A clean `ember <command>` on `jobs` threads: its status, what it reported (the timings line taken
/// out), the threads the timings say it ran on, and each C unit it left.
fn on_threads(dir: &Path, command: &str, jobs: usize, out: &str) -> (bool, String, u64, std::collections::BTreeMap<String, Vec<u8>>) {
    let mut run = Command::new(EMBER);
    run.args([command, "--out-dir", out, "--timings=json", "-j", &jobs.to_string()])
        .current_dir(dir)
        .env(ember_branding::no_incremental_var(), "1")
        .env(ember_branding::no_compile_cache_var(), "1");
    let output = ember_build::output_started(&mut run).expect("ember starts");
    let stderr = String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n").replace(out, "<out>");
    let mut report = String::new();
    let mut threads = 0;
    for line in stderr.lines() {
        if line.starts_with("{\"") && line.contains("\"timings\"") {
            let timings: serde_json::Value = serde_json::from_str(line).expect("timings are JSON");
            threads = timings["jobs"].as_u64().expect("the timings name the threads");
        } else {
            report.push_str(line);
            report.push('\n');
        }
    }
    let units = std::fs::read_dir(dir.join(out).join("debug").join("c"))
        .map(|entries| {
            entries
                .flatten()
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(unit_extension()))
                .map(|entry| (entry.file_name().to_string_lossy().into_owned(), std::fs::read(entry.path()).expect("a unit reads")))
                .collect()
        })
        .unwrap_or_default();
    (output.status.success(), report, threads, units)
}

/// `[BLD-7]` — the front end's per-body analyses run on `-j` threads (by default one per physical
/// core): on one thread and on several, a check reports the same diagnostics in the same order and
/// a build writes the same C, byte for byte.
#[test]
fn a_package_checks_and_builds_alike_on_one_thread_and_on_several() {
    let dir = scratch();
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"shapes\"\nversion = \"0.1.0\"\n").expect("manifest");
    write(&dir, &source("geom"), GEOM);
    write(&dir, &source("shapes"), SHAPES);
    write(&dir, &source("extra"), EXTRA);
    // Warnings in several modules and a body check's error, each reported where its body is.
    write(&dir, &source("main"), &MAIN.replace("    p = geom.Pt(3, 4)\n", "    p = geom.Pt(3, 4)\n    xs: Array[int] = [1]\n    ys = xs\n    println(xs.len(), ys.len())\n"));
    let one = on_threads(&dir, "check", 1, "target-one");
    let several = on_threads(&dir, "check", 4, "target-several");
    assert!(!one.0 && one.1.contains("E3040") && one.1.contains("L1001"), "{}", one.1);
    assert_eq!((one.0, &one.1), (several.0, &several.1), "a check's report on one thread and on four");
    assert_eq!((one.2, several.2), (1, 4), "the threads the timings name");

    write(&dir, &source("main"), MAIN);
    let one = on_threads(&dir, "build", 1, "target-one");
    let several = on_threads(&dir, "build", 4, "target-several");
    assert!(one.0 && several.0, "{}\n{}", one.1, several.1);
    assert_eq!(one.1, several.1, "a build's report on one thread and on four");
    assert!(!one.3.is_empty() && one.3.keys().eq(several.3.keys()), "the C units: {:?} {:?}", one.3.keys(), several.3.keys());
    for (name, text) in &one.3 {
        assert!(several.3[name] == *text, "`{name}` differs between one thread and four");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

const ITEMS_LIB: &str = r#"
pub fn offset(x: int) -> int:
    return x + 1

pub fn twice[F: fn(int) -> int](f: F, x: int) -> int:
    return f(f(x))

pub fn doubled(xs: Span[int]) -> int:
    return xs.iter().map(fn(x: ref int) -> int => x * 2).sum()

pub fn shifted(x: int) -> int:
    f = offset
    return twice(f, x) + f(x)
"#;

const ITEMS_MAIN: &str = r#"
import lib

fn main():
    xs = [1, 2, 3]
    f = lib.offset
    k = fn(x: int) -> int => x * 3
    total = xs.iter().map(fn(x: ref int) -> int => lib.offset(x)).sum()
    println(f(1), lib.twice(f, 2), lib.shifted(1), lib.twice(k, 1), total, lib.doubled(xs.as_span()))
"#;

/// ODR-278, `[BLD-1]` — a function's and a lambda's own types are kept with a module's checked form
/// by the symbol that names the function, and the generics a kept module made for them (`twice`
/// given `lib.offset` or a lambda, `map` given a lambda) are made again by the same symbols: after each edit,
/// every C unit is a clean build's, and the program prints what the clean build's prints.
#[test]
fn function_items_are_kept_across_builds_as_a_clean_build_makes_them() {
    let dir = scratch();
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"items\"\nversion = \"0.1.0\"\n").expect("manifest");
    write(&dir, &source("lib"), ITEMS_LIB);
    write(&dir, &source("main"), ITEMS_MAIN);
    let first = build(&dir, "target-incremental", false);
    assert!(first.ok, "{}", first.report);
    let edits = [
        ("a body the items name", ITEMS_LIB.replace("return x + 1", "return x + 10"), source("lib")),
        ("the importer's body", ITEMS_MAIN.replace("x * 3", "x * 4"), source("main")),
        ("a lambda added before the others", ITEMS_LIB.replace("pub fn doubled", "pub fn tripled(xs: Span[int]) -> int:\n    return xs.iter().map(fn(x: ref int) -> int => x * 3).sum()\n\npub fn doubled"), source("lib")),
    ];
    for (step, text, file) in edits {
        write(&dir, &file, &text);
        let incremental = build(&dir, "target-incremental", false);
        let clean = build(&dir, "target-clean", true);
        assert!(incremental.ok && clean.ok, "{step}:\n{}\n{}", incremental.report, clean.report);
        assert_eq!(incremental.report, clean.report, "{step}: what the builds report");
        assert_eq!(incremental.units.keys().collect::<Vec<_>>(), clean.units.keys().collect::<Vec<_>>(), "{step}: the C units");
        for (name, text) in &incremental.units {
            assert!(clean.units[name] == *text, "{step}: `{name}` is not the clean build's");
        }
        assert_eq!(runs(&dir, "target-incremental"), runs(&dir, "target-clean"), "{step}: what the programs print");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

const UNSTATED_HEADER: &str = "#include <stddef.h>\nsize_t strlen(const char* s);\n";

const UNSTATED_HELPER: &str = r#"
from std.ffi import c_char
import c "unstated.h"

pub fn measure(pointer: *c_char) -> int:
    unsafe:  # SAFETY(ffi): the caller's text is NUL-terminated
        return c.strlen(pointer) as int
"#;

const UNSTATED_MAIN: &str = r#"
from std.ffi import c_char
import c "unstated.h"
import helper

fn main():
    text: Array[c_char] = [104, 105, 0]
    pointer = text.as_span().as_ptr()
    n: usize = 0
    unsafe:  # SAFETY(ffi): `text` is alive and NUL-terminated
        n = c.strlen(pointer)
    println(n, helper.measure(pointer))
"#;

/// `[UNS-7]` (verdicts 125, 134), `[BLD-1]`, D-931 — `L3015` for a C function nothing says the callers
/// of is reported once in a program, at its first call by place (the file built first), with a note
/// naming each other call; whichever modules a check keeps, it reports what a clean one does, and
/// with the first call taken out the warning is at the other, which no note names any more.
#[test]
fn a_c_function_called_with_nothing_stated_is_warned_of_once_whatever_is_kept() {
    let dir = scratch();
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"unstated\"\nversion = \"0.1.0\"\n").expect("manifest");
    std::fs::write(dir.join("src").join("unstated.h"), UNSTATED_HEADER).expect("header");
    write(&dir, &source("helper"), UNSTATED_HELPER);
    write(&dir, &source("main"), UNSTATED_MAIN);
    let helper_call = format!("{}:7:16", source("helper"));
    let steps = [
        ("the first check", None, true),
        ("nothing edited", None, true),
        ("the module of the first call", Some((source("main"), UNSTATED_MAIN.replace("println(n, ", "println(n + 0, "))), true),
        ("the other module", Some((source("helper"), UNSTATED_HELPER.replace("as int\n", "as int + 0\n"))), true),
        ("the first call taken out", Some((source("main"), UNSTATED_MAIN.replace("        n = c.strlen(pointer)\n", "        n = 2\n"))), false),
    ];
    for (step, edit, two_calls) in steps {
        if let Some((file, text)) = edit {
            write(&dir, &file, &text);
        }
        let incremental = check(&dir, "target-incremental", false);
        let clean = check(&dir, "target-clean", true);
        assert!(incremental.ok && clean.ok, "{step}:\n{}\n{}", incremental.report, clean.report);
        assert_eq!(incremental.report, clean.report, "{step}: what the checks report");
        assert_eq!(clean.report.matches("warning[L3015]").count(), 1, "{step}: one `L3015` for `strlen`\n{}", clean.report);
        let notes: Vec<&str> = clean.report.lines().filter(|line| line.contains("= note: also called at ")).collect();
        if two_calls {
            assert!(notes.len() == 1 && notes[0].ends_with(&helper_call), "{step}: the note names the helper's call\n{}", clean.report);
        } else {
            assert!(notes.is_empty() && clean.report.contains(&format!("{}:7:16", source("helper"))), "{step}: the warning is at the helper's call\n{}", clean.report);
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

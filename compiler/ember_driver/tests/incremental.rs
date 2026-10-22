//! `[BLD-1]`, `[BLD-2]`, `[BLD-8]` — the module as the unit of front-end caching and the function as
//! the unit of checking within it: a package checked clean and checked again after each of a run
//! of edits reports exactly what a clean check of it reports, while checking again only the
//! functions an edit reaches.

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
        .filter(|name| ["geom.em", "shapes.em", "extra.em", "main.em"].contains(&name.as_str()))
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
    std::fs::write(dir.join("ember.toml"), "[package]\nname = \"shapes\"\nversion = \"0.1.0\"\n").expect("manifest");
    write(&dir, "geom.em", GEOM);
    write(&dir, "shapes.em", SHAPES);
    write(&dir, "extra.em", EXTRA);
    write(&dir, "main.em", MAIN);
    let geom_only = vec!["geom.em".to_string()];
    let symbols = |names: &[&str]| -> Vec<String> { names.iter().map(|name| name.to_string()).collect() };

    let all = vec!["extra.em", "geom.em", "main.em", "shapes.em"];
    assert_eq!(agree(&dir, "the first check", true).0, all, "the first check checks every module");
    assert_eq!(agree(&dir, "nothing edited", true), (vec![], vec![]), "nothing edited, nothing checked again");

    // A body: that function alone; its callers use no fact of it that changed.
    let geom = GEOM.replace("acc * 31 + i + 1", "acc * 37 + i + 2");
    write(&dir, "geom.em", &geom);
    assert_eq!(agree(&dir, "a body", true), (geom_only.clone(), symbols(&["em_4geom4work"])));

    // A body above a generic its importer instantiates: the rest of the file, the generic and the
    // instance kept with the importer among it, moves; only the edited function is checked.
    let geom = geom.replace("return x + 1", "return x + 1000");
    write(&dir, "geom.em", &geom);
    assert_eq!(agree(&dir, "a body above a generic", true), (geom_only.clone(), symbols(&["em_4geom6offset"])));

    // A body that changes what a caller's borrows read of it: its callers in its module and in its
    // importers are checked again, and nothing else.
    let geom = geom.replace("pub fn first(a: str, b: str) -> str:\n    return a", "pub fn first(a: str, b: str) -> str:\n    return b");
    write(&dir, "geom.em", &geom);
    assert_eq!(
        agree(&dir, "a borrow fact", true),
        (vec!["geom.em".to_string(), "shapes.em".to_string()], symbols(&["em_4geom4pick", "em_4geom5first", "em_6shapes5label"]))
    );
    assert_eq!(agree(&dir, "nothing edited after it", true), (vec![], vec![]), "what was checked again is kept");

    // A signature in a leaf: the leaf and its importer, whole.
    write(&dir, "extra.em", &EXTRA.replace("(x: int) -> int", "(x: i64) -> i64"));
    assert_eq!(agree(&dir, "a leaf's signature", true).0, vec!["extra.em", "main.em"]);

    // A field of a widely imported type: every importer.
    write(&dir, "geom.em", &geom.replace("    pub y: int\n", "    pub y: int\n    pub z: int = 0\n"));
    assert_eq!(agree(&dir, "a type", true).0, vec!["geom.em", "main.em", "shapes.em"]);

    // A subclass overriding a virtual method: every module sees the class hierarchy.
    write(&dir, "main.em", &format!("{MAIN}\nclass Square(shapes.Shape):\n    override fn area(self) -> int:\n        return self.size * self.size\n"));
    assert_eq!(agree(&dir, "a subclass", true).0, all);

    write(&dir, "main.em", MAIN);
    assert_eq!(agree(&dir, "the subclass removed", true).0, all);

    // An error in one module: the kept modules' warnings are reported as a clean check reports them —
    // a type error stops a clean check before it checks bodies (`L1001` is a body check's), an
    // error a body check finds does not.
    write(&dir, "main.em", &MAIN.replace("extra.leaf_value(3)", "extra.leaf_value(\"three\")"));
    assert_eq!(agree(&dir, "a type error", false).0, vec!["main.em"]);
    let errored = check(&dir, "target-incremental", false);
    assert!(errored.report.contains("E2020") && !errored.report.contains("L1001"), "{}", errored.report);
    write(&dir, "main.em", &MAIN.replace("    p = geom.Pt(3, 4)\n", "    p = geom.Pt(3, 4)\n    xs: Array[int] = [1]\n    ys = xs\n    println(xs.len(), ys.len())\n"));
    assert_eq!(agree(&dir, "a body check's error", false).0, vec!["main.em"]);
    let errored = check(&dir, "target-incremental", false);
    assert!(errored.report.contains("E3040") && errored.report.contains("L1001"), "{}", errored.report);

    // The error mended, `main` as it was: a check that found an error keeps nothing, so the record
    // `main` had before the errors holds, and nothing is checked again.
    write(&dir, "main.em", MAIN);
    assert_eq!(agree(&dir, "the error mended", true), (vec![], vec![]));
    let _ = std::fs::remove_dir_all(&dir);
}

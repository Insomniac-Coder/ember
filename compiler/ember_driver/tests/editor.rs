//! `[IDE-3]`, `[IDE-4]`, `[IDE-6]` — the compiler as an editor uses it (`ember_driver::session`).

use std::path::PathBuf;

use ember_driver::session::{Kind, Position, Session};

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

/// A file with an error of each stage: a line that does not parse, a name that resolves to
/// nothing, an expression with no type; and code around them that is fine.
const BROKEN: &str = "\
struct Point:
    x: f64
    y: f64

    fn norm(self) -> f64:
        return self.x * self.x + self.y * self.y

fn parse_error() -> int:
    total = (1 +
    return 2

fn unresolved() -> int:
    missing = nowhere(3)
    count = 4
    return count

fn untypable() -> int:
    flag = true + 1
    steps: int = 5
    return steps

fn fine(p: Point) -> f64:
    scale = 2.0
    return p.norm() * scale

fn main():
    origin = Point(0.0, 0.0)
    println(fine(origin))
";

fn at(line: u32, column: u32) -> Position {
    Position { line, column }
}

fn file(name: &str, text: &str) -> (test_directories::TemporaryDirectory, PathBuf) {
    let dir = temporary_directory(name);
    let path = dir.join(format!("{name}.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&path, text).expect("the file is written");
    (dir, path)
}

/// `[IDE-3]` — a file with a parse error, an unresolvable name and an untypable expression is
/// still checked whole: each is reported, each becomes a placeholder that satisfies everything
/// (nothing else is reported because of one), and hover, completion and symbols answer
/// everywhere in it, in the broken functions too.
#[test]
fn queries_keep_working_in_a_file_with_errors_of_every_stage() {
    let (_dir, path) = file("broken", BROKEN);
    let mut session = Session::new();

    let problems = session.problems(&path);
    let errors: Vec<(u32, String)> = problems.iter().filter(|p| p.error).map(|p| (p.start.line, p.code.clone().unwrap_or_default())).collect();
    assert_eq!(
        errors,
        [(9, "E0101".to_string()), (10, "E0107".to_string()), (13, "E1010".to_string()), (18, "E2020".to_string())],
        "the unclosed `(` and the `return` it swallowed; the unresolvable name; the untypable `+`; and nothing that follows from them: {problems:#?}"
    );

    // In the broken functions, after their errors.
    assert_eq!(hover(&mut session, &path, 14, 5), "count: i64");
    assert_eq!(hover(&mut session, &path, 19, 5), "steps: i64");
    // The placeholders.
    assert_eq!(hover(&mut session, &path, 13, 5), "missing: <error>");
    assert_eq!(hover(&mut session, &path, 18, 5), "flag: <error>");
    // And in the rest: a parameter, a method call, a declaration, a local.
    assert_eq!(hover(&mut session, &path, 24, 12), "p: ref Point");
    assert_eq!(hover(&mut session, &path, 24, 14), "fn norm(self: ref Point) -> f64");
    assert_eq!(hover(&mut session, &path, 5, 8), "fn norm(self) -> f64");
    assert_eq!(hover(&mut session, &path, 27, 5), "origin: Point");

    // Completion: a value's fields and methods, and the names in scope.
    let line = BROKEN.lines().position(|l| l == "    return p.norm() * scale").expect("the line") as u32 + 1;
    let members: Vec<(String, Kind)> = session.complete(&path, at(line, 14)).into_iter().map(|c| (c.label, c.kind)).collect();
    for wanted in [("x", Kind::Field), ("y", Kind::Field), ("norm", Kind::Method)] {
        assert!(members.contains(&(wanted.0.to_string(), wanted.1)), "{wanted:?} in {members:?}");
    }
    let names: Vec<String> = session.complete(&path, at(24, 5)).into_iter().map(|c| c.label).collect();
    for wanted in ["scale", "p", "Point", "unresolved", "main"] {
        assert!(names.contains(&wanted.to_string()), "{wanted} in {names:?}");
    }
    // What is already typed of a name narrows it.
    let narrowed: Vec<String> = session.complete(&path, at(24, 25)).into_iter().map(|c| c.label).collect();
    assert_eq!(narrowed, ["scale"], "`return sc|`");

    // Symbols: every declaration, the broken ones too, with their members.
    let symbols = session.symbols(&path);
    let listed: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(listed, ["Point", "parse_error", "unresolved", "untypable", "fine", "main"]);
    let point: Vec<(&str, Kind)> = symbols[0].children.iter().map(|s| (s.name.as_str(), s.kind)).collect();
    assert_eq!(point, [("x", Kind::Field), ("y", Kind::Field), ("norm", Kind::Method)]);
    assert_eq!(symbols[4].detail, "fn fine(p: Point) -> f64");
    assert_eq!((symbols[4].start.line, symbols[4].start.column), (22, 4));
}

fn hover(session: &mut Session, path: &std::path::Path, line: u32, column: u32) -> String {
    session.hover(path, at(line, column)).map(|h| h.text).unwrap_or_else(|| panic!("nothing at {line}:{column}"))
}

/// A program whose compile-time value and build need the C compiler, and with a borrow error.
const NEEDS_A_BUILD: &str = "\
fn table(n: int) -> Array[int]:
    out: Array[int] = []
    for i in range(n):
        out.push(i * i)
    return out

static TABLE: Array[int] = table(4)

struct Q:
    arr: Array[int]

fn first(mut h: Q) -> ref mut int:
    return ref mut h.arr[0]

fn main():
    h = Q(arr=[1, 2])
    r = first(h)
    h.arr = [3]
    println(r, TABLE)
";

/// `[IDE-4]` — a query is answered from parsing, names and types alone: nothing is built (no
/// output appears beside the file), no compile-time value is evaluated (that would build and run
/// a program with the C compiler) and the borrow checker does not run, so its error — which
/// `ember check` reports — neither appears among the query's problems nor stops an answer.
#[test]
fn queries_need_no_code_generation_and_no_c_compiler() {
    let (dir, path) = file("unbuilt", NEEDS_A_BUILD);
    let before: Vec<_> = std::fs::read_dir(&dir).expect("the directory").map(|e| e.expect("an entry").file_name()).collect();
    let mut session = Session::new();
    assert_eq!(hover(&mut session, &path, 7, 8), "static TABLE: Array[int] = table(4)");
    assert_eq!(hover(&mut session, &path, 17, 5), "r: ref mut i64");
    assert_eq!(hover(&mut session, &path, 16, 5), "h: Q");
    assert!(session.problems(&path).is_empty(), "{:#?}", session.problems(&path));
    assert_eq!(session.symbols(&path).len(), 5);
    let after: Vec<_> = std::fs::read_dir(&dir).expect("the directory").map(|e| e.expect("an entry").file_name()).collect();
    assert_eq!(before, after, "a query wrote nothing");

    // The borrow error is there, for `ember check` to report.
    let checked = std::process::Command::new(env!("CARGO_BIN_EXE_ember"))
        .args(["check", &path.to_string_lossy(), "--out-dir", &dir.join("out").to_string_lossy()])
        .output()
        .expect("ember runs");
    assert!(String::from_utf8_lossy(&checked.stderr).contains("error[E3021]"), "{}", String::from_utf8_lossy(&checked.stderr));
}

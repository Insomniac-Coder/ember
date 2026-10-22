//! `[IMP-11]` — the type checker and the borrow checker, fuzzed through `ember check`. Each test
//! generates a fixed number of programs from a fixed seed (`EMBER_FUZZ_CASES` multiplies the
//! number, `EMBER_FUZZ_SEED` changes the seed, `EMBER_FUZZ_TRACE` prints each case), each
//! function of a program an independent case.
//!
//! The type checker's oracle is the generator's own typing: every function it writes is well
//! typed, so `ember check` must accept it; and a copy with one declaration given a value of
//! another type must be rejected there, with `E2020`, and nowhere else. The borrow checker's is a
//! model of `[BRW-1]`, `[BRW-2]` and `[OWN-3]` for straight-line code: a loan of an array lives
//! from the borrow to the last use of what holds it, and an access conflicting with a live loan
//! (or using a moved array) is an error. Each generated function is one the model accepts, or
//! one with a single access the model rejects; the checker must agree on each, at that line. No
//! input may end the compiler with an internal error.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

#[path = "support/temporary_directory.rs"]
mod test_directories;
use test_directories::temporary_directory;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

/// SplitMix64.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.next() % 100 < percent
    }
}

fn seed() -> u64 {
    std::env::var("EMBER_FUZZ_SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(0x0e4b_e2f0_9d11_0003)
}

fn cases(base: usize) -> usize {
    base * std::env::var("EMBER_FUZZ_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(1)
}

/// One diagnostic of `ember check --json`: its code, severity and primary line.
#[derive(Debug)]
struct Reported {
    code: String,
    error: bool,
    line: usize,
    /// Shown when a case fails.
    #[allow(dead_code)]
    message: String,
}

/// `ember check --json` on each file, side by side. Panics on an internal error.
fn check_all(dir: &Path, files: &[PathBuf]) -> Vec<Vec<Reported>> {
    let next = AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).min(files.len().max(1));
    let mut results: Vec<(usize, Vec<Reported>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some(file) = files.get(i) else { break done };
                        done.push((i, check(dir, file)));
                    }
                })
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().expect("a check thread")).collect()
    });
    results.sort_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, reported)| reported).collect()
}

fn check(dir: &Path, file: &Path) -> Vec<Reported> {
    let output = ember_build::output_started(
        Command::new(EMBER).args(["check", "--json", "--out-dir"]).arg(dir.join("out")).arg(file).current_dir(dir),
    )
    .expect("ember starts");
    let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(
        !text.contains("panicked") && !text.contains("internal compiler error") && output.status.code().is_some_and(|c| c <= 1),
        "[IMP-11] the compiler failed on {}:\n{text}\n{}",
        file.display(),
        std::fs::read_to_string(file).unwrap_or_default()
    );
    text.lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line.trim()).ok())
        .filter(|d| d.get("severity").is_some())
        .map(|d| Reported {
            code: d["code"].as_str().unwrap_or("none").to_string(),
            error: d["severity"] == "error",
            line: d["labels"]
                .as_array()
                .and_then(|labels| labels.iter().find(|label| label["primary"] == true))
                .and_then(|label| label["span"]["line_start"].as_u64())
                .unwrap_or(0) as usize,
            message: d["message"].as_str().unwrap_or("").to_string(),
        })
        .collect()
}

/// A program made of independent functions: each one's text, the line it starts on, and what
/// the oracle expects of it (`None`: accepted; `Some((line, codes))`: its first error at that line
/// with one of those codes, and no error outside it).
struct Program {
    text: String,
    functions: Vec<(usize, usize, Option<(usize, &'static [&'static str])>)>,
}

/// Checks each program, all at once, against its oracle.
fn run_cases(label: &str, programs: Vec<Program>) {
    let dir = temporary_directory(label);
    let files: Vec<PathBuf> = programs
        .iter()
        .enumerate()
        .map(|(index, program)| {
            let path = dir.join(format!("case{index}.{}", ember_branding::SOURCE_EXT));
            std::fs::write(&path, &program.text).expect("the case is written");
            path
        })
        .collect();
    // `EMBER_FUZZ_TRACE` prints each case and what the oracle expects of each function.
    if std::env::var_os("EMBER_FUZZ_TRACE").is_some() {
        for (program, file) in programs.iter().zip(&files) {
            eprintln!("{}: {:?}
{}", file.display(), program.functions, program.text);
        }
    }
    let results = check_all(&dir, &files);
    for ((program, file), reported) in programs.iter().zip(&files).zip(results) {
        let errors: Vec<&Reported> = reported.iter().filter(|r| r.error).collect();
        for &(first, last, expected) in &program.functions {
            let inside: Vec<&&Reported> = errors.iter().filter(|r| (first..=last).contains(&r.line)).collect();
            let function = program.text.lines().skip(first - 1).take(last + 1 - first).collect::<Vec<_>>().join("\n");
            match expected {
                None => assert!(
                    inside.is_empty(),
                    "[IMP-11] {label}: {} rejected a function the oracle accepts:\n{function}\n{inside:#?}",
                    file.display()
                ),
                Some((line, codes)) => {
                    let first_error = inside.iter().min_by_key(|r| r.line);
                    assert!(
                        first_error.is_some_and(|r| r.line == line && codes.contains(&r.code.as_str())),
                        "[IMP-11] {label}: {} should reject line {line} with {codes:?}:\n{function}\nreported: {inside:#?}",
                        file.display()
                    );
                }
            }
        }
        let placed = |r: &&Reported| program.functions.iter().any(|&(first, last, _)| (first..=last).contains(&r.line));
        let stray: Vec<&&Reported> = errors.iter().filter(|r| !placed(r)).collect();
        assert!(stray.is_empty(), "[IMP-11] {label}: {} has errors in no function:\n{stray:#?}\n{}", file.display(), program.text);
    }
}

// ---------------------------------------------------------------------------
// The type checker
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ty {
    Int,
    Float,
    Bool,
    Str,
    Ints,
    Struct(usize),
}

struct Typed<'a> {
    rng: &'a mut Rng,
    /// Each struct's fields (all scalars, so the struct is `Copy`).
    structs: Vec<Vec<Ty>>,
    /// Each earlier function's parameters and result.
    functions: Vec<(Vec<Ty>, Ty)>,
    /// The names in scope: name, type, whether it may be assigned and pushed to.
    scope: Vec<(String, Ty, bool)>,
    /// The arrays a `for` is iterating, which may not be pushed to.
    iterated: Vec<String>,
    names: usize,
    lines: Vec<String>,
    /// The lines `v: T = e` declares, with their type, which a mutation may give another type.
    typed_lines: Vec<(usize, Ty)>,
}

impl Typed<'_> {
    fn ty_name(&self, ty: Ty) -> String {
        match ty {
            Ty::Int => "int".into(),
            Ty::Float => "float".into(),
            Ty::Bool => "bool".into(),
            Ty::Str => "String".into(),
            Ty::Ints => "Array[int]".into(),
            Ty::Struct(k) => format!("S{k}"),
        }
    }

    fn any_ty(&mut self) -> Ty {
        match self.rng.below(6 + usize::from(!self.structs.is_empty())) {
            0 | 1 => Ty::Int,
            2 => Ty::Float,
            3 => Ty::Bool,
            4 => Ty::Str,
            5 => Ty::Ints,
            _ => Ty::Struct(self.rng.below(self.structs.len())),
        }
    }

    fn fresh(&mut self) -> String {
        self.names += 1;
        format!("v{}", self.names)
    }

    fn var_of(&mut self, ty: Ty) -> Option<String> {
        let found: Vec<String> = self.scope.iter().filter(|(_, t, _)| *t == ty).map(|(n, _, _)| n.clone()).collect();
        (!found.is_empty()).then(|| found[self.rng.below(found.len())].clone())
    }

    /// An expression of type `ty`. A `String` or an `Array` is never read from a name except
    /// where it is borrowed (`[OWN-3]`: assigning one moves it).
    fn expr(&mut self, ty: Ty, depth: usize) -> String {
        let leaf = depth == 0 || self.rng.chance(30);
        let copy = !matches!(ty, Ty::Str | Ty::Ints);
        if copy && self.rng.chance(if leaf { 50 } else { 15 }) && let Some(name) = self.var_of(ty) {
            return name;
        }
        let d = depth.saturating_sub(1);
        // A call of an earlier function with this result.
        if !leaf && self.rng.chance(15) {
            let callees: Vec<usize> = (0..self.functions.len()).filter(|&f| self.functions[f].1 == ty).collect();
            if !callees.is_empty() {
                let f = callees[self.rng.below(callees.len())];
                let params = self.functions[f].0.clone();
                let args: Vec<String> = params.iter().map(|&p| self.argument(p, d)).collect();
                return format!("f{f}({})", args.join(", "));
            }
        }
        match ty {
            Ty::Int => match if leaf { 0 } else { self.rng.below(9) } {
                0 => self.rng.below(100).to_string(),
                1 => format!("({} {} {})", self.expr(Ty::Int, d), ["+", "-", "*"][self.rng.below(3)], self.expr(Ty::Int, d)),
                2 => format!("({} {} {})", self.expr(Ty::Int, d), ["//", "%"][self.rng.below(2)], 1 + self.rng.below(9)),
                3 => format!("-({})", self.expr(Ty::Int, d)),
                4 => format!("{}.len()", self.argument(Ty::Ints, d)),
                5 => format!("({} if {} else {})", self.expr(Ty::Int, d), self.expr(Ty::Bool, d), self.expr(Ty::Int, d)),
                6 => format!("({} as int)", self.expr(Ty::Float, d)),
                7 => self.field(Ty::Int, d).unwrap_or_else(|| "7".into()),
                // Every array the generator makes has an element.
                _ => format!("{}[0]", self.argument(Ty::Ints, d)),
            },
            Ty::Float => match if leaf { 0 } else { self.rng.below(6) } {
                0 => format!("{}.{}", self.rng.below(100), self.rng.below(10)),
                1 => format!("({} {} {})", self.expr(Ty::Float, d), ["+", "-", "*"][self.rng.below(3)], self.expr(Ty::Float, d)),
                2 => format!("({} / {}.5)", self.expr(Ty::Float, d), self.rng.below(9)),
                3 => format!("({} as float)", self.expr(Ty::Int, d)),
                4 => self.field(Ty::Float, d).unwrap_or_else(|| "0.25".into()),
                _ => format!("({} if {} else {})", self.expr(Ty::Float, d), self.expr(Ty::Bool, d), self.expr(Ty::Float, d)),
            },
            Ty::Bool => match if leaf { 0 } else { self.rng.below(8) } {
                0 => ["true", "false"][self.rng.below(2)].into(),
                1 => format!("({} {} {})", self.expr(Ty::Int, d), ["<", "<=", "==", "!=", ">", ">="][self.rng.below(6)], self.expr(Ty::Int, d)),
                2 => format!("({} {} {})", self.expr(Ty::Float, d), ["<", ">"][self.rng.below(2)], self.expr(Ty::Float, d)),
                3 => format!("(not {})", self.expr(Ty::Bool, d)),
                4 => format!("({} {} {})", self.expr(Ty::Bool, d), ["and", "or"][self.rng.below(2)], self.expr(Ty::Bool, d)),
                5 => format!("{}.is_empty()", self.argument(Ty::Ints, d)),
                6 => self.field(Ty::Bool, d).unwrap_or_else(|| "true".into()),
                _ => format!("({} < {} <= {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            },
            Ty::Str => match if leaf { 0 } else { self.rng.below(2) } {
                0 => format!("\"s{}\"", self.rng.below(10)),
                _ => {
                    let (a, b) = (self.expr(Ty::Int, d), self.expr(Ty::Bool, d));
                    let s = self.var_of(Ty::Str).map(|s| format!(" {{{s}}}")).unwrap_or_default();
                    format!("f\"{{{a}}} {{{b}}}{s}\"")
                }
            },
            Ty::Ints => {
                let items: Vec<String> = (0..self.rng.below(3) + 1).map(|_| self.expr(Ty::Int, d)).collect();
                format!("[{}]", items.join(", "))
            }
            Ty::Struct(k) => {
                let fields = self.structs[k].clone();
                let values: Vec<String> = fields.iter().enumerate().map(|(i, &f)| format!("f{i}={}", self.expr(f, d))).collect();
                format!("S{k}({})", values.join(", "))
            }
        }
    }

    /// A value passed to a parameter (borrowed): a name of any type will do.
    fn argument(&mut self, ty: Ty, depth: usize) -> String {
        if self.rng.chance(50) && let Some(name) = self.var_of(ty) {
            return name;
        }
        self.expr(ty, depth)
    }

    fn field(&mut self, ty: Ty, depth: usize) -> Option<String> {
        let owners: Vec<(usize, usize)> = (0..self.structs.len())
            .flat_map(|k| self.structs[k].iter().enumerate().filter(|(_, f)| **f == ty).map(move |(i, _)| (k, i)).collect::<Vec<_>>())
            .collect();
        if owners.is_empty() {
            return None;
        }
        let (k, i) = owners[self.rng.below(owners.len())];
        Some(format!("{}.f{i}", self.argument(Ty::Struct(k), depth)))
    }

    fn line(&mut self, indent: usize, text: String) {
        self.lines.push(format!("{}{text}", "    ".repeat(indent)));
    }

    fn block(&mut self, indent: usize, depth: usize) {
        let mark = self.scope.len();
        for _ in 0..self.rng.below(4) + 1 {
            self.statement(indent, depth);
        }
        self.scope.truncate(mark);
    }

    fn statement(&mut self, indent: usize, depth: usize) {
        let compound = depth > 0;
        match self.rng.below(if compound { 11 } else { 6 }) {
            0 | 1 => {
                let ty = self.any_ty();
                let value = self.expr(ty, 3);
                let name = self.fresh();
                // `[TXT-9]`: a literal makes a `String` only where one is expected; bare, it is a `str`.
                let written = if value.starts_with('"') { ": String" } else { "" };
                self.line(indent, format!("{name}{written} = {value}"));
                self.scope.push((name, ty, true));
            }
            2 => {
                let ty = self.any_ty();
                let value = self.expr(ty, 3);
                let name = self.fresh();
                let written = self.ty_name(ty);
                self.line(indent, format!("{name}: {written} = {value}"));
                self.typed_lines.push((self.lines.len(), ty));
                self.scope.push((name, ty, true));
            }
            3 => {
                let assignable: Vec<(String, Ty)> = self.scope.iter()
                    .filter(|(_, t, mutable)| *mutable && !matches!(t, Ty::Str | Ty::Ints))
                    .map(|(n, t, _)| (n.clone(), *t)).collect();
                if let Some((name, ty)) = (!assignable.is_empty()).then(|| assignable[self.rng.below(assignable.len())].clone()) {
                    if ty == Ty::Int && self.rng.chance(50) {
                        let value = self.expr(Ty::Int, 2);
                        let op = ["+=", "-=", "*="][self.rng.below(3)];
                        self.line(indent, format!("{name} {op} {value}"));
                    } else {
                        let value = self.expr(ty, 2);
                        self.line(indent, format!("{name} = {value}"));
                    }
                } else {
                    self.line(indent, "pass".into());
                }
            }
            4 => {
                let pushable: Vec<String> = self.scope.iter()
                    .filter(|(n, t, mutable)| *t == Ty::Ints && *mutable && !self.iterated.contains(n))
                    .map(|(n, _, _)| n.clone()).collect();
                if pushable.is_empty() {
                    let value = self.expr(Ty::Str, 2);
                    self.line(indent, format!("println({value})"));
                } else {
                    let name = pushable[self.rng.below(pushable.len())].clone();
                    let value = self.expr(Ty::Int, 2);
                    self.line(indent, format!("{name}.push({value})"));
                }
            }
            5 => {
                let ty = [Ty::Int, Ty::Float, Ty::Bool][self.rng.below(3)];
                let value = self.expr(ty, 2);
                self.line(indent, format!("println(f\"{{{value}}}\")"));
            }
            6 | 7 => {
                let condition = self.expr(Ty::Bool, 2);
                self.line(indent, format!("if {condition}:"));
                self.block(indent + 1, depth - 1);
                if self.rng.chance(50) {
                    self.line(indent, "else:".into());
                    self.block(indent + 1, depth - 1);
                }
            }
            8 => {
                let count = self.expr(Ty::Int, 1);
                let name = self.fresh();
                self.line(indent, format!("for {name} in 0..{count}:"));
                let mark = self.scope.len();
                self.scope.push((name, Ty::Int, false));
                self.block(indent + 1, depth - 1);
                self.scope.truncate(mark);
            }
            9 => {
                let Some(array) = self.var_of(Ty::Ints) else { return self.line(indent, "pass".into()) };
                let name = self.fresh();
                self.line(indent, format!("for {name} in {array}:"));
                let mark = self.scope.len();
                self.scope.push((name, Ty::Int, false));
                self.iterated.push(array);
                self.block(indent + 1, depth - 1);
                self.iterated.pop();
                self.scope.truncate(mark);
            }
            _ => {
                let counter = self.fresh();
                self.line(indent, format!("{counter} = 0"));
                let bound = 1 + self.rng.below(4);
                self.line(indent, format!("while {counter} < {bound}:"));
                self.line(indent + 1, format!("{counter} += 1"));
                self.block(indent + 1, depth - 1);
            }
        }
    }
}

/// A program of well-typed functions, and for each function `v: T = e` lines that a mutation
/// may give a value of another type.
fn typed_program(rng: &mut Rng, mutate: bool) -> Program {
    let mut typed = Typed { rng, structs: Vec::new(), functions: Vec::new(), scope: Vec::new(), iterated: Vec::new(),
        names: 0, lines: Vec::new(), typed_lines: Vec::new() };
    for k in 0..typed.rng.below(3) {
        let fields: Vec<Ty> = (0..typed.rng.below(3) + 1).map(|_| [Ty::Int, Ty::Float, Ty::Bool][typed.rng.below(3)]).collect();
        typed.line(0, "@derive(Copy)".into());
        typed.line(0, format!("struct S{k}:"));
        for (i, &f) in fields.iter().enumerate() {
            let name = typed.ty_name(f);
            typed.line(1, format!("f{i}: {name}"));
        }
        typed.line(0, String::new());
        typed.structs.push(fields);
    }
    let mut functions = Vec::new();
    for f in 0..typed.rng.below(5) + 2 {
        let params: Vec<Ty> = (0..typed.rng.below(4)).map(|_| typed.any_ty()).collect();
        let ret = typed.any_ty();
        typed.scope = params.iter().enumerate().map(|(i, &t)| (format!("p{i}"), t, false)).collect();
        typed.typed_lines.clear();
        let signature: Vec<String> = params.iter().enumerate().map(|(i, &t)| format!("p{i}: {}", typed.ty_name(t))).collect();
        let ret_name = typed.ty_name(ret);
        typed.line(0, format!("fn f{f}({}) -> {ret_name}:", signature.join(", ")));
        let first = typed.lines.len();
        typed.block(1, 2);
        let value = typed.expr(ret, 3);
        typed.line(1, format!("return {value}"));
        let last = typed.lines.len();
        let mut expected = None;
        if mutate && !typed.typed_lines.is_empty() && typed.rng.chance(60) {
            let (line, ty) = typed.typed_lines[typed.rng.below(typed.typed_lines.len())];
            // A value of another type that no literal rule makes fit (`[LEX-16]`: an integer
            // literal is a fine `float`).
            let wrong = loop {
                let other = typed.any_ty();
                if other != ty && !(ty == Ty::Float && other == Ty::Int) {
                    break other;
                }
            };
            let value = typed.expr(wrong, 2);
            let text = &typed.lines[line - 1];
            let at = text.find(" = ").expect("a declaration");
            typed.lines[line - 1] = format!("{} = {value}", &text[..at]);
            expected = Some((line, &["E2020"][..]));
        }
        typed.line(0, String::new());
        functions.push((first, last, expected));
        typed.functions.push((params, ret));
    }
    typed.line(0, "fn main():".into());
    typed.line(1, "pass".into());
    Program { text: typed.lines.join("\n") + "\n", functions }
}

#[test]
fn well_typed_programs_are_accepted_and_one_wrong_type_is_found_where_it_is() {
    let mut rng = Rng(seed() ^ 0x7e9c);
    let programs: Vec<Program> = (0..cases(60)).map(|case| typed_program(&mut rng, case % 2 == 1)).collect();
    run_cases("typeck-fuzz", programs);
}

// ---------------------------------------------------------------------------
// The borrow checker
// ---------------------------------------------------------------------------

/// One straight-line operation on the arrays `a0`…`a2` and the views and references made of them.
#[derive(Clone, Copy, Debug)]
enum Op {
    /// `vN: Span[int] = aK` — a shared loan of `aK`, held by `vN`.
    View { holder: usize, array: usize },
    /// `mN: ref mut int = ref mut aK[0]` — a mutable loan of `aK`.
    Lend { holder: usize, array: usize },
    /// A read through the holder.
    Use { holder: usize },
    /// `aK.len()`, `aK[0]`: a read of the array.
    Read { array: usize },
    /// `aK.push(…)`: a write.
    Push { array: usize },
    /// `aK = […]`: a write that also gives a moved array a value again.
    Assign { array: usize },
    /// `bN = aK`: a move.
    Move { array: usize, to: usize },
}

const ARRAYS: usize = 3;

/// What the model says of `ops[at]`: `None` if it is allowed, else the codes its error has.
fn verdict(ops: &[Op], at: usize) -> Option<&'static [&'static str]> {
    // Each holder's array, whether its loan is mutable, and whether it is used after `at`.
    let live = |array: usize, mutable_only: bool| {
        ops[..at].iter().any(|op| {
            let (holder, of, mutable) = match *op {
                Op::View { holder, array } => (holder, array, false),
                Op::Lend { holder, array } => (holder, array, true),
                _ => return false,
            };
            of == array && (mutable || !mutable_only) && ops[at + 1..].iter().any(|later| matches!(later, Op::Use { holder: h } if *h == holder))
        })
    };
    // Whether `array` was moved and not given a value since.
    let moved = |array: usize| {
        let mut moved = false;
        for op in &ops[..at] {
            match *op {
                Op::Move { array: a, .. } if a == array => moved = true,
                Op::Assign { array: a } if a == array => moved = false,
                _ => {}
            }
        }
        moved
    };
    const CONFLICT: &[&str] = &["E3020", "E3021", "E3022"];
    const MOVED: &[&str] = &["E3040"];
    match ops[at] {
        Op::Use { .. } => None,
        Op::Read { array } | Op::View { array, .. } => {
            if moved(array) { Some(MOVED) } else if live(array, true) { Some(CONFLICT) } else { None }
        }
        Op::Lend { array, .. } | Op::Push { array } | Op::Move { array, .. } => {
            if moved(array) { Some(MOVED) } else if live(array, false) { Some(CONFLICT) } else { None }
        }
        Op::Assign { array } => live(array, false).then_some(CONFLICT),
    }
}

fn op_text(op: Op, rng: &mut Rng) -> String {
    match op {
        Op::View { holder, array } => format!("v{holder}: Span[int] = a{array}"),
        Op::Lend { holder, array } => format!("m{holder}: ref mut int = ref mut a{array}[0]"),
        // Which holder it is the text knows by its letter: views are even, references odd.
        Op::Use { holder } if holder % 2 == 0 => format!("println(v{holder}[0] + v{holder}.len())"),
        Op::Use { holder } => if rng.chance(50) { format!("m{holder} += 1") } else { format!("println(m{holder})") },
        Op::Read { array } => if rng.chance(50) { format!("println(a{array}.len())") } else { format!("println(a{array}[0])") },
        Op::Push { array } => format!("a{array}.push({})", rng.below(100)),
        Op::Assign { array } => format!("a{array} = [{}, 1, 2]", rng.below(100)),
        Op::Move { array, to } => format!("b{to} = a{array}"),
    }
}

/// A random operation over what exists so far.
fn random_op(rng: &mut Rng, ops: &[Op], holders: &mut usize, moves: &mut usize) -> Op {
    let array = rng.below(ARRAYS);
    let made: Vec<usize> = ops.iter().filter_map(|op| match op {
        Op::View { holder, .. } | Op::Lend { holder, .. } => Some(*holder),
        _ => None,
    }).collect();
    match rng.below(9) {
        0 | 1 => {
            *holders += 1;
            let holder = *holders * 2;
            Op::View { holder, array }
        }
        2 => {
            *holders += 1;
            let holder = *holders * 2 + 1;
            Op::Lend { holder, array }
        }
        3 | 4 if !made.is_empty() => Op::Use { holder: made[rng.below(made.len())] },
        5 => Op::Read { array },
        6 => Op::Push { array },
        7 => Op::Assign { array },
        8 => {
            *moves += 1;
            Op::Move { array, to: *moves }
        }
        _ => Op::Read { array },
    }
}

/// A function the model accepts, or one with a single access it rejects.
fn borrow_function(rng: &mut Rng, name: &str, reject: bool) -> (Vec<String>, Option<(usize, &'static [&'static str])>) {
    let (mut holders, mut moves) = (0, 0);
    let mut ops: Vec<Op> = Vec::new();
    // Build an accepted sequence: an operation the model rejects is not kept. A later use can
    // make an earlier access conflict, so the whole sequence is judged again each time.
    for _ in 0..rng.below(10) + 4 {
        for _ in 0..8 {
            let op = random_op(rng, &ops, &mut holders, &mut moves);
            ops.push(op);
            if (0..ops.len()).all(|at| verdict(&ops, at).is_none()) {
                break;
            }
            ops.pop();
        }
    }
    let mut rejected = None;
    if reject {
        // One access placed where the model rejects it and where it changes no other verdict.
        for _ in 0..40 {
            let at = rng.below(ops.len() + 1);
            let array = rng.below(ARRAYS);
            let op = match rng.below(4) {
                0 => Op::Read { array },
                1 => Op::Push { array },
                2 => {
                    holders += 1;
                    Op::Lend { holder: holders * 2 + 1, array }
                }
                _ => {
                    holders += 1;
                    Op::View { holder: holders * 2, array }
                }
            };
            let mut candidate = ops.clone();
            candidate.insert(at, op);
            let others_fine = (0..candidate.len()).filter(|&i| i != at).all(|i| verdict(&candidate, i).is_none());
            if let (Some(codes), true) = (verdict(&candidate, at), others_fine) {
                ops = candidate;
                rejected = Some((at, codes));
                break;
            }
        }
    }
    let mut lines = vec![format!("fn {name}():")];
    for array in 0..ARRAYS {
        lines.push(format!("    a{array}: Array[int] = [{}, {}, 3]", rng.below(9), rng.below(9)));
    }
    let first_op_line = lines.len() + 1;
    for &op in &ops {
        let text = op_text(op, rng);
        lines.push(format!("    {text}"));
    }
    lines.push("    pass".into());
    (lines, rejected.map(|(at, codes)| (first_op_line + at, codes)))
}

#[test]
fn the_borrow_checker_agrees_with_a_model_of_loans_and_moves() {
    let mut rng = Rng(seed() ^ 0xb0b0);
    let programs: Vec<Program> = (0..cases(40))
        .map(|_| {
            let mut text = String::new();
            let mut functions = Vec::new();
            let mut line = 1;
            for f in 0..8 {
                let reject = rng.chance(50);
                let (lines, expected) = borrow_function(&mut rng, &format!("g{f}"), reject);
                let first = line;
                for l in &lines {
                    let _ = writeln!(text, "{l}");
                }
                line += lines.len();
                functions.push((first, line - 1, expected.map(|(at, codes)| (first - 1 + at, codes))));
                text.push('\n');
                line += 1;
            }
            text.push_str("fn main():\n    pass\n");
            Program { text, functions }
        })
        .collect();
    run_cases("borrowck-fuzz", programs);
}

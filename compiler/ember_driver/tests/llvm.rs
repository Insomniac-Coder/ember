//! `[CG-LL-1]`..`[CG-LL-3]` — the LLVM backend's module, read as text (`--emit llvm-ir`): what the
//! IR says about checks, floating point, vectors, attributes and debug information. Nothing here
//! builds or runs a program; the conformance tree runs them (`EMBER_BACKEND=llvm`).

#[path = "support/temporary_directory.rs"]
mod temporary_directory;

use std::process::Command;

use temporary_directory::temporary_directory;

const EMBER: &str = env!("CARGO_BIN_EXE_ember");

/// The module `source` makes in `profile`, as `ember build --backend llvm --emit llvm-ir` prints it.
fn ir(source: &str, profile: &str) -> String {
    let dir = temporary_directory("llvm");
    let path = dir.join(format!("program.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&path, source).expect("the program is written");
    let output = Command::new(EMBER)
        .args(["build", "--backend", "llvm", "--emit", "llvm-ir", "--profile", profile])
        .arg(&path)
        .current_dir(&*dir)
        .output()
        .expect("ember starts");
    assert!(
        output.status.success(),
        "the LLVM backend refused the program:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The text of the function `symbol`'s definition.
fn function<'a>(ir: &'a str, symbol: &str) -> &'a str {
    let header = format!("@{symbol}(");
    let mut start = None;
    let mut at = 0;
    for line in ir.split_inclusive('\n') {
        if line.starts_with("define ") && line.contains(&header) {
            start = Some(at);
            break;
        }
        at += line.len();
    }
    let start = start.unwrap_or_else(|| panic!("`{symbol}` is defined"));
    let end = ir[start..].find("\n}\n").map_or(ir.len(), |end| start + end + 3);
    &ir[start..end]
}

const ARITHMETIC: &str = "\
@noinline\nfn mix(a: i64, b: i64, c: i32, d: u8) -> i64:
    return (a + b) * (a - b) + (c as i64) // 3 + (d + 1) as i64

@noinline
fn shifted(a: i64, s: i64) -> i64:
    return (a << s) + (a >> s)

fn main():
    println(mix(7, 2, -5, 4), shifted(3, 2))
";

/// `[CG-LL-2]` — checked arithmetic is LLVM's overflow intrinsics, each overflow the runtime's
/// panic; no instruction claims what was not proved (`nsw`, `nuw`, `inbounds`, `noundef`), and a
/// shift's amount is masked to its width.
#[test]
fn cg_ll_2_checked_arithmetic_is_the_overflow_intrinsics_and_nothing_is_assumed() {
    let ir = ir(ARITHMETIC, "debug");
    for intrinsic in ["llvm.sadd.with.overflow.i64", "llvm.ssub.with.overflow.i64", "llvm.smul.with.overflow.i64", "llvm.uadd.with.overflow.i8"] {
        assert!(ir.contains(intrinsic), "no `{intrinsic}` in the module");
    }
    assert!(ir.contains("@ember_panic_overflow"), "an overflow is not the runtime's panic");
    for claim in [" nsw ", " nuw ", " inbounds ", "noundef", " exact "] {
        assert!(!ir.contains(claim), "the module assumes `{}`", claim.trim());
    }
    let shifted = function(&ir, "em_7shifted");
    assert!(shifted.contains("and i64"), "a shift's amount is not masked:\n{shifted}");
}

/// `[CG-LL-2]`, `[TYP-9]`, `[DET-5]` — floating point is IEEE's, one rounding per operation, with
/// no fast-math flag but in a function that asks for one: `@fastmath` relaxes its own body only,
/// and never assumes NaN or infinity away (that would be poison).
#[test]
fn cg_ll_2_floating_point_relaxes_only_where_a_function_asks() {
    let ir = ir(
        "\
@noinline\nfn strict(a: f64, b: f64) -> f64:
    return a * b + a

@fastmath
@noinline
fn relaxed(a: f64, b: f64) -> f64:
    return a * b + a

fn main():
    println(strict(1.5, 2.0), relaxed(1.5, 2.0))
",
        "release",
    );
    let strict = function(&ir, "em_6strict");
    assert!(strict.contains("fmul double") && strict.contains("fadd double"), "{strict}");
    for flag in ["fast", "reassoc", "contract", "nnan", "ninf", "nsz", "arcp", "afn"] {
        assert!(!strict.contains(flag), "a strict function's IR has `{flag}`:\n{strict}");
    }
    let relaxed = function(&ir, "em_7relaxed");
    assert!(relaxed.contains("reassoc") && relaxed.contains("contract"), "{relaxed}");
    assert!(!relaxed.contains("nnan") && !relaxed.contains("ninf"), "`@fastmath` assumed finite values:\n{relaxed}");
}

/// `[CG-LL-2]`, `[SIMD-1]` — `std.simd`'s vectors are LLVM's vector types, an operation one vector
/// instruction.
#[test]
fn cg_ll_2_simd_vectors_are_llvm_vectors() {
    let ir = ir(
        "\
from std.simd import f32x8, i32x4

fn main():
    a = f32x8.splat(1.5)
    b = a * a + a
    c = i32x4.splat(3) + i32x4.splat(4)
    println(b.reduce_add(), c.reduce_add())
",
        "debug",
    );
    assert!(ir.contains("fmul <8 x float>") && ir.contains("fadd <8 x float>"), "the vectors are not LLVM vectors");
    assert!(ir.contains("llvm.sadd.with.overflow.v4i32"), "a vector's checked `+` is not the vector intrinsic");
    assert!(ir.contains("llvm.vector.reduce.fadd.v8f32"), "a float reduction is not LLVM's ordered one");
}

/// `[CG-LL-3]` — facts the types prove, given as attributes and never assumed: a reference
/// parameter is `nonnull`; a result passed through memory is a fresh slot (`noalias sret`).
#[test]
fn cg_ll_3_references_are_nonnull_and_results_in_memory_are_noalias() {
    let ir = ir(
        "\
struct Big:
    a: i64
    b: i64
    c: i64

fn total(x: ref Big) -> i64:
    return x.a + x.b + x.c

fn make(n: i64) -> Big:
    return Big(a=n, b=n + 1, c=n + 2)

fn main():
    big = make(4)
    println(total(ref big))
",
        "debug",
    );
    let total = function(&ir, "em_5total");
    assert!(total.lines().next().is_some_and(|header| header.contains("ptr nonnull")), "{total}");
    let make = function(&ir, "em_4make");
    assert!(make.lines().next().is_some_and(|header| header.contains("ptr noalias sret(")), "{make}");
}

/// `[CG-LL-3]`, `[THR-14]` — the IR in the shape LLVM optimises as it does the runtime's C: every
/// function names the CPU and features Clang gives C (without them a link-time optimisation
/// compiles the IR for the linker's default CPU, never inlines a runtime inline function into the
/// program, and calls a library for a 16-byte compare-exchange), and an
/// element's address is one scaled `getelementptr`, which LLVM's unrolling counts as C's indexing.
#[test]
fn cg_ll_3_functions_name_the_runtimes_cpu_and_elements_are_scaled_addresses() {
    let ir = ir(
        "\
@noinline
fn total(xs: ref Array[i64]) -> i64:
    sum = 0
    for i in 0..xs.len():
        sum += xs[i]
    return sum

fn main():
    xs = [1, 2, 3]
    println(total(ref xs))
",
        "release",
    );
    let total = function(&ir, "em_5total");
    assert!(
        total.lines().next().is_some_and(|header| header.contains("\"target-cpu\"=\"x86-64\"") && header.contains("\"target-features\"=\"+cx16\"")),
        "{total}"
    );
    assert!(total.contains("getelementptr [8 x i8], ptr"), "an element's address is not a scaled `getelementptr`:\n{total}");
    assert!(!total.contains("mul i64"), "an element's offset is multiplied apart:\n{total}");
}

/// `[SIMD-7]` — a read-only byte sum whose checks the MIR proved away is the runtime's
/// `sum_bytes`, as the C backend selects it: signed bytes and a subtracted total too. A total
/// whose address is taken keeps its loop.
#[test]
fn simd_7_a_byte_sum_is_the_runtimes_sum_bytes() {
    let ir = ir(
        "\
@noinline
fn totals(xs: Span[u8], ys: Span[i8]) -> (int, int):
    a = 9
    b = -17
    for x in xs:
        a += x as int
    for y in ys:
        b -= y as int
    return (a, b)

fn main():
    xs: Array[u8] = []
    ys: Array[i8] = []
    for i in 0..256:
        xs.push(i as u8)
        ys.push((i - 128) as i8)
    (a, b) = totals(xs[0..256], ys[0..256])
    println(a, b)
",
        "release",
    );
    let totals = function(&ir, "em_6totals");
    assert_eq!(totals.matches("@ember_ll_sum_bytes(").count(), 2, "{totals}");
    assert!(totals.contains("zeroext false)") && totals.contains("zeroext true)"), "the sums do not say which bytes are signed:\n{totals}");
}

/// `[CG-LL-3]`, `[RT-4]` — debug information in every profile: each function a subprogram named
/// by its symbol, each instruction its Ember line; CodeView on Windows, DWARF elsewhere.
#[test]
fn cg_ll_3_debug_information_names_functions_and_lines() {
    for profile in ["debug", "release"] {
        let ir = ir(ARITHMETIC, profile);
        assert!(ir.contains("!DISubprogram(name: \"em_3mix\""), "{profile}: no subprogram for `mix`");
        assert!(ir.contains("!DILocation(line: 3,"), "{profile}: no location on the line of `mix`'s `return`");
        assert!(ir.contains("!\"Debug Info Version\", i32 3"));
        let format = if cfg!(windows) { "!\"CodeView\"" } else { "!\"Dwarf Version\"" };
        assert!(ir.contains(format), "{profile}: not {format}");
        let mix = function(&ir, "em_3mix");
        assert!(mix.lines().next().is_some_and(|header| header.contains("!dbg !")), "{mix}");
    }
}

/// `[CG-LL-1]`, `[TST-3]` — `ember test --backend llvm`: the test binary the LLVM backend builds
/// runs each test alone, by its number, as the C backend's does; a panic, an expected panic and a
/// return each read as the runner expects.
#[test]
fn cg_ll_1_ember_test_runs_the_tests_the_llvm_backend_built() {
    let package = temporary_directory("llvm-test");
    std::fs::write(package.join(ember_branding::MANIFEST), "[package]\nname = \"tested\"\nkind = \"bin\"\n").expect("the manifest is written");
    std::fs::create_dir_all(package.join("src")).expect("src is made");
    std::fs::write(
        package.join("src").join(ember_branding::source_file("main")),
        "\
fn add(a: int, b: int) -> int:
    return a + b

@test
fn adds():
    assert(add(2, 3) == 5)

@test
fn fails():
    assert(add(2, 2) == 5)

@test
@should_panic
fn panics_as_it_should():
    panic(\"boom\")

fn main():
    println(\"main is not a test\")
",
    )
    .expect("the program is written");
    let output = Command::new(EMBER).args(["test", "--backend", "llvm"]).current_dir(&*package).output().expect("ember starts");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let said = format!("{stdout}{}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.status.success(), "a failing test passed:\n{said}");
    for line in ["running 3 tests", "test adds ... ok", "test fails ... FAILED", "test panics_as_it_should ... ok", "2 passed; 1 failed"] {
        assert!(stdout.contains(line), "missing `{line}`:\n{said}");
    }
    assert!(!stdout.contains("main is not a test"), "the test binary ran `main`:\n{said}");
}

/// `[CG-LL-1]` — the backend is chosen by `--backend` (or `[build] backend`), and one the build
/// does not know is refused by name.
#[test]
fn cg_ll_1_an_unknown_backend_is_refused() {
    let dir = temporary_directory("llvm");
    let path = dir.join(format!("program.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&path, "fn main():\n    println(1)\n").expect("the program is written");
    let output = Command::new(EMBER).args(["build", "--backend", "gcc-jit", "--emit", "llvm-ir"]).arg(&path).output().expect("ember starts");
    assert!(!output.status.success());
    let said = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(said.contains("gcc-jit"), "{said}");
}

/// ODR-278, `[CLO-1]`, `[STD-19]` — a lambda that captures nothing has a type of its own, which
/// `map`'s generic argument keeps: the adapter is made for the lambda, holds nothing for it, and
/// its `next` calls the lambda directly. No callable value is made or called through (in `debug`,
/// where Ember inlines nothing, ODR-259, the call is there to read).
#[test]
fn clo_1_a_lambda_in_an_adapter_is_called_directly() {
    let ir = ir(
        "\
fn main():
    xs = [1, 2, 3]
    it = xs.iter().map(fn(x: ref int) -> int => x * 2)
    total = 0
    for v in it:
        total += v
    println(total)
",
        "debug",
    );
    let symbol_of = |needle: &str| -> String {
        let line = ir.lines().find(|line| line.starts_with("define ") && line.contains(needle)).unwrap_or_else(|| panic!("a function `{needle}`:\n{ir}"));
        let at = line.find('@').expect("a symbol") + 1;
        line[at..at + line[at..].find('(').expect("its parameters")].to_string()
    };
    let lambda = symbol_of("closure_4main");
    let next = function(&ir, &symbol_of("3Map4next"));
    assert!(next.contains(&format!("@{lambda}(")), "the adapter calls its lambda directly:\n{next}");
    assert!(!ir.contains("fn_value"), "no callable value:\n{ir}");
}

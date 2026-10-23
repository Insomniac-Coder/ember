//! `[CG-LL-1]`..`[CG-LL-3]` — the LLVM backend's module, read as text (`--emit llvm-ir`): what the
//! IR says about checks, floating point, vectors, attributes and debug information; and the builds
//! only this backend makes (`ember test`'s binary, profile-guided optimisation's two). The
//! conformance tree runs the programs (`EMBER_BACKEND=llvm`).

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

/// `[CG-LL-3]` (ODR-275, ODR-276) — a reference parameter of an Ember function is
/// `dereferenceable` for its referent's size (no callee can end what a borrowed parameter points
/// into: the caller's loan keeps it, RC-5, HEAP-5, EXC-2), and a shared one is `readonly` unless
/// its referent holds a `Cell` or an `UnsafeCell`, which every std type changed through a shared
/// reference keeps its state in (`RefCell`, `Mutex`, `Atomic`, and a struct holding one). An
/// export's caller is foreign: none of it there.
#[test]
fn cg_ll_3_references_are_dereferenceable_and_shared_ones_without_a_cell_readonly() {
    let ir = ir(
        "\
from std.sync import Atomic, Mutex, MemoryOrder

struct Pair:
    a: i64
    b: i64

struct Stats:
    hits: Atomic[i64]
    name: i64

@noinline
fn sum(p: ref Pair) -> i64:
    return p.a + p.b

@noinline
fn bump(mut p: Pair):
    p.a += 1

@noinline
fn poke(cell: ref Cell[i64]):
    cell.set(cell.get() + 1)

@noinline
fn grow(cell: ref RefCell[i64]):
    with g = cell.borrow_mut():
        g += 1

@noinline
fn tick(stats: ref Stats):
    stats.hits.fetch_add(1, MemoryOrder.Relaxed)

@noinline
fn locked(m: ref Mutex[i64]) -> i64:
    with g = m.lock():
        return g

@export(\"pair_total\")
fn pair_total(p: Pair) -> i64:
    return p.a + p.b

fn main():
    p = Pair(a=1, b=2)
    bump(p)
    c = Cell(1i64)
    poke(ref c)
    r = RefCell(1i64)
    grow(ref r)
    stats = Stats(hits=Atomic(0i64), name=0)
    tick(ref stats)
    m = Mutex(3i64)
    println(sum(ref p), c.get(), locked(ref m), pair_total(p))
",
        "release",
    );
    let header = |symbol: &str| function(&ir, symbol).lines().next().unwrap_or_default().to_string();
    assert!(header("em_3sum").contains("ptr nonnull dereferenceable(16) readonly"), "{}", header("em_3sum"));
    let bump = header("em_4bump");
    assert!(bump.contains("ptr nonnull dereferenceable(16)") && !bump.contains("readonly"), "{bump}");
    for (symbol, size) in [("em_4poke", 8), ("em_4grow", 32), ("em_4tick", 16), ("em_6locked", 16)] {
        let header = header(symbol);
        assert!(header.contains(&format!("ptr nonnull dereferenceable({size})")) && !header.contains("readonly"), "{header}");
    }
    let export = header("pair_total");
    assert!(!export.contains("dereferenceable") && !export.contains("readonly"), "{export}");
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

/// `[CG-LL-3]`, `[BLD-6]` (ODR-274) — profile-guided optimisation: `--pgo-generate` builds a program
/// instrumented to write its raw profile into the profile's `pgo` folder as it ends; `--pgo-use`
/// with that folder merges it (`llvm-profdata merge`) and optimises with it, and the profile holds
/// what the run did (`work`'s entry count is the times `main` called it). A profile that is not one
/// stops the build at Clang, one not there before it; the command line wins over the manifest's
/// `[profiles.<name>] pgo`, as `--profile` does; the C backend refuses the flags by name.
#[test]
fn cg_ll_3_profile_guided_optimisation_records_and_uses_a_profile() {
    let dir = temporary_directory("llvm-pgo");
    std::fs::write(dir.join(ember_branding::MANIFEST), "[package]\nname = \"warm\"\n\n[profiles.release]\npgo = \"use\"\npgo_data = \"absent.profdata\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src").join(ember_branding::source_file("main")), "\
@noinline
fn work(n: int) -> int:
    total = 0
    for i in 0..n:
        total += i % 7
    return total

fn main():
    sum = 0
    for k in 0..3:
        sum += work(1000 + k)
    println(sum)
").unwrap();
    let ember = |args: &[&str]| Command::new(EMBER).args(args).current_dir(&*dir).output().expect("ember starts");
    let said = |output: &std::process::Output| format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    // The manifest asks for a profile that is not there: the command line's ask wins.
    let generated = ember(&["build", "--backend", "llvm", "--profile", "release", "--pgo-generate"]);
    assert!(generated.status.success(), "the instrumented build failed:\n{}", said(&generated));
    let record = std::fs::read_to_string(dir.join("target/release/inspect/main.build.json")).expect("the build record");
    assert!(record.contains("\"pgo\":{\"generate\""), "{record}");
    let exe = dir.join("target/release/bin").join(format!("main{}", std::env::consts::EXE_SUFFIX));
    let ran = Command::new(&exe).output().expect("the instrumented program runs");
    assert_eq!(String::from_utf8_lossy(&ran.stdout).trim(), "9003", "{}", said(&ran));
    let raw: Vec<_> = std::fs::read_dir(dir.join("target/release/pgo")).expect("the profile folder")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "profraw"))
        .collect();
    assert!(!raw.is_empty(), "the instrumented run wrote no raw profile");
    // The manifest's `pgo_data` is not there: refused before Clang.
    let missing = ember(&["build", "--backend", "llvm", "--profile", "release"]);
    assert!(!missing.status.success() && said(&missing).contains("no recorded profile"), "{}", said(&missing));
    let used = ember(&["build", "--backend", "llvm", "--profile", "release", "--pgo-use", "target/release/pgo", "--out-dir", "used"]);
    assert!(used.status.success(), "the optimised build failed:\n{}", said(&used));
    let record = std::fs::read_to_string(dir.join("used/release/inspect/main.build.json")).expect("the build record");
    assert!(record.contains("\"pgo\":{\"use\""), "{record}");
    let merged = dir.join("used/release/pgo/main.profdata");
    assert!(merged.is_file(), "no merged profile");
    // What the run did: `work`'s loop turned 1000 + 1001 + 1002 times, and it was entered three.
    let clang = ember_build::Toolchain::detect(Some("clang")).expect("Clang").compiler_path().to_path_buf();
    let clang = clang.canonicalize().unwrap_or(clang);
    let tool = clang.with_file_name(format!("llvm-profdata{}", std::env::consts::EXE_SUFFIX));
    let shown = Command::new(&tool).args(["show", "--all-functions", "--counts"]).arg(&merged).output().expect("llvm-profdata runs");
    let text = String::from_utf8_lossy(&shown.stdout);
    let at = text.find("em_4work:").unwrap_or_else(|| panic!("`work` is not in the profile:\n{text}"));
    let work: Vec<&str> = text[at..].lines().take(5).collect();
    assert!(work.iter().any(|line| line.trim().starts_with("Block counts: [3003, 3,")), "{text}");
    // A profile that is not one stops the build at Clang.
    std::fs::write(dir.join("broken.profdata"), "not a profile").unwrap();
    let broken = ember(&["build", "--backend", "llvm", "--profile", "release", "--pgo-use", "broken.profdata", "--out-dir", "broken"]);
    assert!(!broken.status.success(), "a broken profile was used:\n{}", said(&broken));
    // The C backend has no profile-guided optimisation of its own.
    let c = ember(&["build", "--backend", "c", "--pgo-generate"]);
    assert!(!c.status.success() && said(&c).contains("LLVM backend"), "{}", said(&c));
}

/// `[FFI-9]`, `[FFI-10]` — `extern "vectorcall"` on Windows x64: a floating-point aggregate of up to
/// four members goes in vector registers while they last (here after scalars took theirs, and not
/// at all once six are taken), a larger or mixed aggregate behind a pointer, such a result comes
/// back in registers, and the symbols are MSVC's (`vc_scale@@16`). The program calls C compiled by
/// the backend's own compiler, and both backends print the same.
#[cfg(windows)]
#[test]
fn ffi_9_vectorcall_passes_floating_point_aggregates_in_vector_registers() {
    let dir = temporary_directory("llvm-vectorcall");
    std::fs::write(dir.join("vc.h"), "/* The functions are declared by the program, with their convention. */\n").unwrap();
    std::fs::write(dir.join("vc.c"), "\
#include \"vc.h\"
typedef struct { double a, b; } hva2;
typedef struct { float x, y, z; } hfa3;
typedef struct { double a, b, c, d, e; } big5;
double __vectorcall vc_scale(double x, double factor) { return x * factor; }
double __vectorcall vc_hva2(hva2 h) { return h.a + 10 * h.b; }
double __vectorcall vc_after(int a, hva2 h, double b, hfa3 f) { return a + 10 * h.a + 100 * h.b + 1000 * b + 10000 * f.x + 100000 * f.y + 1000000 * f.z; }
hva2 __vectorcall vc_give(double a) { hva2 h = { a, a * 2 }; return h; }
double __vectorcall vc_many(double a, double b, double c, double d, double e, double f, double g, hva2 h) {
    return a + 2 * b + 3 * c + 4 * d + 5 * e + 6 * f + 7 * g + 8 * h.a + 9 * h.b;
}
double __vectorcall vc_big(big5 v) { return v.a + 2 * v.b + 3 * v.c + 4 * v.d + 5 * v.e; }
").unwrap();
    let source = dir.join(format!("main.{}", ember_branding::SOURCE_EXT));
    std::fs::write(&source, "\
import c \"./vc.c\"

struct Hva2:
    a: f64
    b: f64

struct Hfa3:
    x: f32
    y: f32
    z: f32

struct Big5:
    a: f64
    b: f64
    c: f64
    d: f64
    e: f64

unsafe extern \"vectorcall\":
    safe fn vc_scale(x: f64, factor: f64) -> f64
    safe fn vc_hva2(h: Hva2) -> f64
    safe fn vc_after(a: i32, h: Hva2, b: f64, f: Hfa3) -> f64
    safe fn vc_give(a: f64) -> Hva2
    safe fn vc_many(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: Hva2) -> f64
    safe fn vc_big(v: Big5) -> f64

fn main():
    h = Hva2(a=1.0, b=2.0)
    println(vc_scale(2.0, 3.0), vc_hva2(h))
    println(vc_after(1, h, 2.0, Hfa3(x=3.0, y=4.0, z=5.0)))
    given = vc_give(1.5)
    println(given.a, given.b)
    println(vc_many(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, h))
    println(vc_big(Big5(a=1.0, b=1.0, c=1.0, d=1.0, e=1.0)))
").unwrap();
    let mut outputs = Vec::new();
    for backend in ["c", "llvm"] {
        let out = format!("out-{backend}");
        let built = Command::new(EMBER).args(["build", "--backend", backend, "--out-dir", &out]).arg(&source).current_dir(&*dir).output().expect("ember starts");
        assert!(built.status.success(), "{backend}: {}{}", String::from_utf8_lossy(&built.stdout), String::from_utf8_lossy(&built.stderr));
        if backend == "llvm" {
            let ir = Command::new(EMBER).args(["build", "--backend", "llvm", "--emit", "llvm-ir"]).arg(&source).current_dir(&*dir).output().expect("ember starts");
            let ir = String::from_utf8_lossy(&ir.stdout).into_owned();
            for declared in [
                "declare x86_vectorcallcc double @\"\\01vc_scale@@16\"(double, double)",
                "declare x86_vectorcallcc double @\"\\01vc_hva2@@16\"({ double, double } inreg)",
                "declare x86_vectorcallcc double @\"\\01vc_after@@48\"(i32, { double, double } inreg, double, { float, float, float } inreg)",
                "declare x86_vectorcallcc { double, double } @\"\\01vc_give@@8\"(double)",
                "declare x86_vectorcallcc double @\"\\01vc_many@@72\"(double, double, double, double, double, double, double, ptr)",
                "declare x86_vectorcallcc double @\"\\01vc_big@@40\"(ptr)",
            ] {
                assert!(ir.contains(declared), "no `{declared}`:\n{ir}");
            }
        }
        let exe = dir.join(&out).join("debug/bin/main.exe");
        let ran = Command::new(&exe).output().expect("the program runs");
        assert!(ran.status.success(), "{backend}: {}", String::from_utf8_lossy(&ran.stderr));
        outputs.push(String::from_utf8_lossy(&ran.stdout).replace("\r\n", "\n"));
    }
    // 1 + 10·1 + 100·2 + 1000·2 + 10000·3 + 100000·4 + 1000000·5; 28 + 8·1 + 9·2.
    let lines: Vec<&str> = outputs[0].lines().collect();
    assert!(lines.len() == 5 && lines[1].starts_with("5432211") && lines[3].starts_with("54"), "the C backend's:\n{}", outputs[0]);
    assert_eq!(outputs[0], outputs[1], "the backends disagree");
}

# Clause TCB-4.1: "The unsafe section lists each `unsafe:` block, `unsafe fn`, `unsafe import c` and
# `unsafe import cpp` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))`
# (`[FFI-26]`) with its note; the C FFI section lists each `safe fn`, `@ffi(no_callbacks)`
# (`[FFI-52]`), `@ffi(safe)` (`[FFI-56]`) and thread contract"
#$ test: compile-pass
#$ rules: TCB-4, FFI-51, FFI-52, FFI-26, FFI-22, FFI-54
# The program a milestone test runs `ember tcb` on (the conformance harness checks no report: INDEX
# §0). The report lists, in the unsafe section, `unsafe import c "./bridge.c"`,
# `@unsafe(export("tcb_tick"))`, the `unsafe extern "C"` block and `give_away`'s `unsafe:` block
# around `into_raw()` (A5-4, which decisions.md makes the listing only), each with its note, and the
# `unsafe fn` `unregister` by its `@safety`; in the C FFI section, the `safe fn`s `register_cb` and
# `sqrt`, `sqrt`'s `@ffi(no_callbacks)`, and `register_cb`'s `threads(creator)`.
# Here the file only compiles.

# SAFETY: `bridge.c` defines one function that adds two numbers, and runs nothing before `main`
unsafe import c "./bridge.c" as b

# SAFETY: nothing else in the process defines `tcb_tick`
@unsafe(export("tcb_tick"))
pub extern "C" fn tick() -> i32:
    return 1

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

struct Point:
    x: int

fn give_away(owns p: owns Point) -> *mut Point:
    # SAFETY: the address goes to C, which ends the point with `ember_rt_object_release`
    unsafe:
        return p.into_raw()

fn main():
    _given = give_away(owns Point(x=1))
    _token = register_cb(owns fn(): println("kept"))
    println(sqrt(4.0))

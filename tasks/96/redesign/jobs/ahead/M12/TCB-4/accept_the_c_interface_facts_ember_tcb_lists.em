# Clause TCB-4.1: "The unsafe section lists each `unsafe:` block, `unsafe fn`, `unsafe import c` and
# `unsafe import cpp` (`[FFI-51]`) and `@unsafe(export(…))` or `@unsafe(export_table(…))`
# (`[FFI-26]`) with its note; the C FFI section lists each `safe fn`, `@ffi(no_callbacks)`
# (`[FFI-52]`), `@ffi(safe)` (`[FFI-56]`) and thread contract"
#$ test: compile-pass
#$ rules: TCB-4, FFI-51, FFI-52, FFI-26, FFI-22
# The program a milestone test runs `ember tcb` on (the harness checks no report, Q5). The report
# lists, in the unsafe section, `unsafe import c "./bridge.c"` and `@unsafe(export("tcb_tick"))`,
# each with its note, and the `unsafe fn` `unregister`; in the C FFI section, the `safe fn`s
# `register_cb` and `sqrt`, `sqrt`'s `@ffi(no_callbacks)`, and `register_cb`'s `threads(creator)`.
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

fn main():
    _token = register_cb(owns fn(): println("kept"))
    println(sqrt(4.0))

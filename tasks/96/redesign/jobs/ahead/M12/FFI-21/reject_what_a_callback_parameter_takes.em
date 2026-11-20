# Clause FFI-21.1: "A function-pointer parameter of a `safe fn` or of an overlay entry carries a
# `callback(…)` word, or the declaration is `E5002`"; "under `callback(retained)` or `callback(once)`
# C keeps the function, so the parameter is an `owns` parameter, `owns cb: fn(P…) -> R`, and its
# argument says `owns` at the call (...), a function written in the call (...) and a named one
# (`register_cb(owns add)`, after which `add` is handed over, `E3040` at a later use) alike, `E3028`
# without it"; "A function that captures anything is `E5040` unless the parameter's contract names a
# `user_data` parameter"; "(`E5041` if none is declared, for `once` too"
#$ test: compile-fail
#$ rules: FFI-21, FFI-11, FN-2a
# `set_timer`'s message is the spec's; the rest is code only (E5002's row: "a `callback(…)`
# parameter written as the C pointer type or without `owns` under `retained` or `once`"). The
# declarations are never linked: the test only compiles.

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    safe fn set_timer(cb: extern "C" fn())    #$ error[E5002]: `set_timer`'s `cb` is a function pointer, and nothing says whether C keeps it or which thread runs it; write `param(cb, callback(retained, threads(any)))`, or `callback(borrowed)` if C runs it only during the call
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn keeps_the_c_type(cb: extern "C" fn(*mut void), ud: *mut void)    #$ error[E5002]
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn keeps_without_owns(cb: fn(), ud: *mut void)    #$ error[E5002]
    @ffi(param(cb, callback(once, user_data=ud), threads(creator)))
    safe fn later(owns cb: once fn(), ud: *mut void)    #$ error[E5041]
    @ffi(param(cb, callback(borrowed)))
    safe fn run_now(cb: fn())

fn main():
    _one = register_cb(fn(): println("one"))    #$ error[E3028]
    add = fn(): println("added")
    _two = register_cb(owns add)
    add()    #$ error[E3040]
    n = 1
    run_now(fn(): println(n))    #$ error[E5040]

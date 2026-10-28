#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-10
#$ profiles: debug, development, release
#$ stdout: 330
#$ stdout: 8 8
# `[FFI-21]` (ODR-245) — a callback with several parameters of its user data's type names the one
# that receives it: `user_data=p@N`, the callback's `N`th parameter, counted from 1 as the C
# signature writes them. The callable a caller passes takes the others, in their order: here an
# `isize` item beside the `isize` context, and a pointer beside the pointer C hands back. The
# trampoline finds the callable at the parameter `@N` names, so a capture is read through it. With
# one parameter of the type, `@N` may still be written.

from std.ffi import c_int

import c "./registry.c" as raw

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(visit, callback(borrowed, user_data=context@2)), link_name="each_pair")
    safe fn each_pair(count: c_int, visit: extern "C" fn(isize, isize, c_int) -> c_int, context: isize) -> c_int
    @ffi(param(pick, callback(borrowed, user_data=right@2)), link_name="both")
    safe fn pick(pick: extern "C" fn(*mut void, *mut void) -> c_int, right: *mut void) -> c_int
    @ffi(param(hook, callback(borrowed, user_data=data@1)), link_name="maybe_call")
    safe fn call_hook(hook: extern "C" fn(*mut void, c_int) -> c_int, data: *mut void, value: c_int) -> c_int

fn main():
    scale: c_int = 100
    println(each_pair(3, fn(item: isize, index: c_int) -> c_int => item as c_int + index * scale))
    base: c_int = 7
    seen = pick(fn(left: *mut void) -> c_int => base + 1)
    println(seen, call_hook(fn(value: c_int) -> c_int => value + base, 1))

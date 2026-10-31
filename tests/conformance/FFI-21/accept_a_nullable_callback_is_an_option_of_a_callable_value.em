#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-23
#$ profiles: debug, development, release
#$ stdout: -1 0
#$ stdout: 15 1
#$ stdout: 0 true 0
#$ stdout: 1 1
#$ stdout: 12 1
#$ stdout: 0
#$ stdout: -1 0
#$ stdout: hook 9
#$ stdout: 9
# `[FFI-21]`, `[FFI-11]` (ODR-253) — a `nullable` callback with user data is an `Option` of a
# callable value (`Option[fn(c_int) -> c_int]`, `once fn` for a `once` one): `None` gives C a null
# function pointer and null user data, and `Some` a trampoline and the value's address, called
# through the value's entry. Borrowed, the value stays where the caller has it; `once` and
# `retained` move it into an allocation of their own, and a `retained` one's token is an `Option`
# too: `None` registered nothing, so nothing is unregistered.

from std.ffi import c_int, ForeignCallback, Retained

import c "./registry.c" as raw

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(hook, callback(borrowed, user_data=data), nullable), link_name="maybe_call")
    safe fn maybe_call(hook: Option[extern "C" fn(*mut void, c_int) -> c_int], data: *mut void, value: c_int) -> c_int
    @ffi(link_name="given")
    safe fn given() -> c_int
    @ffi(param(hook, callback(retained, user_data=data), nullable), destroys(unsubscribe), link_name="subscribe")
    safe fn subscribe(hook: Option[extern "C" fn(*mut void, c_int) -> c_int], data: *mut void) -> c_int
    @safety("callers pass what the C function `unsubscribe` requires")
    unsafe fn unsubscribe(id: c_int)
    @ffi(link_name="fire")
    safe fn fire(value: c_int) -> c_int
    @ffi(link_name="subscribers")
    safe fn subscribers() -> c_int
    @ffi(param(hook, callback(once, user_data=data), nullable), link_name="defer")
    safe fn defer_hook(hook: Option[extern "C" fn(*mut void, c_int) -> c_int], data: *mut void)
    @ffi(link_name="run_deferred")
    safe fn run_deferred(value: c_int) -> c_int

fn main():
    println(maybe_call(None, 5), given())
    k: c_int = 3
    println(maybe_call(Some(fn(value: c_int) -> c_int => value * k), 5), given())
    (none_id, none_token) = subscribe(None)
    println(none_id, none_token.is_none(), given())
    offset: c_int = 10
    (id, token) = subscribe(Some(fn(value: c_int) -> c_int => value + offset))
    println(id - 99, subscribers())
    println(fire(2), subscribers())
    release(token)
    println(subscribers())
    defer_hook(None)
    println(run_deferred(4), given())
    label = "hook".to_string()
    shout = owned fn(value: c_int) -> c_int:
        println(label, value)
        return value
    defer_hook(Some(shout))
    println(run_deferred(9))

fn release(owned token: Option[Retained[ForeignCallback]]):
    pass

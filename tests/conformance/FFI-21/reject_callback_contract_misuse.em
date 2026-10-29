#$ test: compile-fail
#$ rules: FFI-21, FFI-11
#$ profiles: debug
# `[FFI-21]`, `[FFI-11]` — what a callback's contract must say: the parameter is a C function
# pointer, its `user_data=p` names a parameter its function pointer takes back (one of that
# type), a `retained` callback names what unregisters it (`destroys(f)`, `E5041`), each of
# `f`'s parameters is the callback, its user data or a scalar the registering function takes,
# and a malformed word is `E0104`. Without user data, a capturing lambda is `E5040`.

from std.ffi import ForeignCallback

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `unregister` requires")
    fn unregister(handler: extern "C" fn(*mut void, i32), data: *mut void)
    @safety("callers pass what the C function `unregister_badly` requires")
    fn unregister_badly(handler: extern "C" fn(*mut void, i32), data: *mut void, what: *mut i32)

    @ffi(param(handler, callback(borrowed)))
    safe fn call_plain(handler: extern "C" fn(i32) -> i32) -> i32

    @ffi(param(count, callback(borrowed, user_data=data)))
    safe fn not_a_pointer(count: i32, data: *mut void)    #$ error[E5002]: `callback(…)` is written on a C function pointer

    @ffi(param(handler, callback(borrowed, user_data=nowhere)))
    safe fn names_nothing(handler: extern "C" fn(*mut void, i32))    #$ error[E5002]: `user_data=nowhere` names no parameter

    @ffi(param(handler, callback(borrowed, user_data=data)))
    safe fn no_slot(handler: extern "C" fn(i32), data: *mut void)    #$ error[E5002]: takes no `*mut void` for its user data `data`

    @ffi(param(handler, callback(borrowed, user_data=data)))
    safe fn two_slots(handler: extern "C" fn(*mut void, *mut void), data: *mut void)    #$ error[E5002]: takes 2 parameters of the user data's type

    @ffi(param(handler, callback(retained, user_data=data)))
    safe fn keeps(handler: extern "C" fn(*mut void, i32), data: *mut void)    #$ error[E5041]: nothing unregisters it

    @ffi(param(handler, callback(retained, user_data=data)), destroys(unregister_badly))    #$ error[E5002]: `unregister_badly`'s parameter `what: *mut i32` is none of what `keeps_badly` passes it
    safe fn keeps_badly(handler: extern "C" fn(*mut void, i32), data: *mut void)

    @ffi(param(handler, callback(sideways, user_data=data)))    #$ error[E0104]: `sideways` is not how long C keeps a callback
    safe fn sideways(handler: extern "C" fn(*mut void, i32), data: *mut void)

    @ffi(param(handler, callback(borrowed, user_data=data)), unsafe)    #$ error[E5002]: `unsafe` leaves the call unsafe
    safe fn left_unsafe(handler: extern "C" fn(*mut void, i32), data: *mut void)

fn main():
    k: i32 = 5
    println(call_plain(fn(x: i32) -> i32 => x + k))    #$ error[E5040]: found a lambda that captures

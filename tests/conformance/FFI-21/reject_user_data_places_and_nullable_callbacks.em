#$ test: compile-fail
#$ rules: FFI-21, FFI-11, FFI-22
#$ profiles: debug
# `[FFI-21]` (ODR-245) — `user_data=p@N` names the callback's `N`th parameter, counted from 1: a
# place the callback does not have, or one of another type, is `E5002`; `@0` is no place (`E0104`).
# Without `@N`, a callback with several parameters of the user data's type is `E5002`, whose help
# shows the `@N` form. `[FFI-11]` (ODR-246) — `destroys(f)` is given the registering call's own
# parameters and its result: a parameter of `f` that is neither, or a result C does not give, is
# `E5002`. `[FFI-22]` (ODR-253) — a `nullable` callback's `Option` of a callable value: what a
# lambda written in `Some` captures is `Send` unless the contract names a thread, and a callable
# value whose captures are unknown is not (`E7004`); `None` captures nothing. A function that passes
# its own parameter on is checked where it is called, whichever is checked first.

from std.ffi import c_int

class Counter:
    count: i32 = 0

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `forget` requires")
    fn forget(id: c_int)
    @safety("callers pass what the C function `forget_wrongly` requires")
    fn forget_wrongly(id: i64)

    @ffi(param(visit, callback(borrowed, user_data=data@3)))
    safe fn past_the_end(visit: extern "C" fn(*mut void, c_int), data: *mut void)    #$ error[E5002]: parameter 3 of the callback `visit` does not exist: it takes 2

    @ffi(param(visit, callback(borrowed, user_data=data@2)))
    safe fn other_type(visit: extern "C" fn(*mut void, c_int), data: *mut void)    #$ error[E5002]: parameter 2 of the callback `visit` is a `i32`, not the user data's `*mut void`

    @ffi(param(visit, callback(borrowed, user_data=data@0)))    #$ error[E0104]: the callback's parameters are counted from 1
    safe fn no_place(visit: extern "C" fn(*mut void, c_int), data: *mut void)

    @ffi(param(visit, callback(borrowed, user_data=data)))
    safe fn ambiguous(visit: extern "C" fn(*mut void, *mut void), data: *mut void)    #$ error[E5002]: takes 2 parameters of the user data's type
    #$ help: name the one that receives it: `user_data=data@N`, `N` its place in the C signature (1 or 2)

    @ffi(param(visit, callback(retained, user_data=data)), destroys(forget_wrongly))    #$ error[E5002]: `forget_wrongly`'s parameter `id: i64` is none of what `keeps_by_id` passes it
    safe fn keeps_by_id(visit: extern "C" fn(*mut void, c_int), data: *mut void) -> c_int

    @ffi(param(visit, callback(retained, user_data=data)), destroys(forget))    #$ error[E5002]: `forget`'s parameter `id: i32` is none of what `keeps_without_id` passes it
    safe fn keeps_without_id(visit: extern "C" fn(*mut void, c_int), data: *mut void)

    @ffi(param(visit, callback(borrowed, user_data=data), nullable))
    safe fn maybe(visit: Option[extern "C" fn(*mut void, c_int)], data: *mut void)

    @ffi(param(visit, callback(borrowed, user_data=data), nullable, threads(main)))
    safe fn maybe_on_main(visit: Option[extern "C" fn(*mut void, c_int)], data: *mut void)

fn main():
    counter = Counter()
    passes_on(Some(owned fn(x: c_int): println(counter.count + x)))    #$ error[E7004]: captures `counter`, a `Counter`, which is not Send
    passes_on(None)
    passes_on(Some(fn(x: c_int): println(x)))
    n: c_int = 2
    f: fn(c_int) = owned fn(x: c_int): println(x + n)
    maybe(Some(f))    #$ error[E7004]: may hold anything, so it is not Send
    maybe(None)
    maybe(Some(fn(x: c_int): println(x)))
    maybe(Some(owned fn(x: c_int): println(counter.count + x)))    #$ error[E7004]: captures `counter`, a `Counter`, which is not Send
    maybe_on_main(Some(owned fn(x: c_int): println(counter.count + x)))

fn passes_on(hook: Option[fn(c_int)]):
    maybe(hook)

# A lambda's own parameter is no parameter of the function it is written in: one it passes to a
# `nullable` callback is a callable value of unknown captures.
fn wraps(n: c_int, hook: Option[fn(c_int)]):
    pass_it = fn(given: c_int, cb: Option[fn(c_int)]): maybe(cb)    #$ error[E7004]: may hold anything, so it is not Send
    pass_it(n, hook)

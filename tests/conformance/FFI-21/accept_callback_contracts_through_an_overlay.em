#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-23
#$ profiles: debug, development, release
#$ stdout: 330
#$ stdout: -1 0 8 1
#$ stdout: 1 2
#$ stdout: 26 2
#$ stdout: 6 1
#$ stdout: 0
# `[FFI-21]`, `[FFI-11]` — the callback contracts of ODR-245, ODR-246 and ODR-253 through an
# overlay on the header: `user_data=context@2` names the second of two `intptr_t` parameters;
# `callback(borrowed, user_data=data) nullable` takes an `Option` of a callable value; and
# `destroys(unsubscribe)` is given the id `subscribe` returned (its parameter `id` is none of
# `subscribe`'s), which the token keeps: `subscribe` returns the id and the token, and the token's
# end unsubscribes that id. No `unsafe`.

from std.ffi import c_int, ForeignCallback, Retained

import c "./registry.c" with (overlay = "overlays/registry.em") as reg

fn main():
    scale: c_int = 100
    println(reg.each_pair(3, fn(item: isize, index: c_int) -> c_int => item as c_int + index * scale))
    missing = reg.maybe_call(None, 5)
    was = reg.given()
    println(missing, was, reg.maybe_call(Some(fn(value: c_int) -> c_int => value + 7), 1), reg.given())
    (first, one) = reg.subscribe(owned fn(value: c_int) -> c_int => value * scale)
    (second, two) = reg.subscribe(owned fn(value: c_int) -> c_int => value + 4)
    println(second - first, reg.subscribers())
    println(reg.fire(0) + reg.fire(0) + 18, reg.subscribers())
    end(one)
    println(reg.fire(2), reg.subscribers())
    end(two)
    println(reg.subscribers())

fn end(owned token: Retained[ForeignCallback]):
    pass

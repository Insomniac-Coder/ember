#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-23
#$ profiles: debug, development, release
#$ stdout: 0 true -2
#$ stdout: -2
#$ stdout: 7 105
#$ stdout: 207 -1
#$ stdout: 8 8
#$ stdout: 308
# `[FFI-21]`, `[FFI-11]` (ODR-253, ODR-246, ODR-271) — a `nullable` `retained` callback whose
# `destroys(f)` takes one of the registering call's arguments and its result: `unsubscribe_on` gets
# the `channel` the call was given and the id it returned, both kept in the callback's record with
# the trampoline and the user data. `None` registers nothing, so its token is `None` and its end
# unregisters nothing; a `Some` token's end calls `unsubscribe_on(channel, id)`, then ends the
# callable.

from std.ffi import c_int, ForeignCallback, Retained

import c "./channels.c" as raw

unsafe extern "C":
    @ffi(param(hook, callback(retained, user_data=data), nullable), destroys(unsubscribe_on), link_name="subscribe_on")
    safe fn subscribe_on(channel: c_int, hook: Option[extern "C" fn(*mut void, c_int) -> c_int], data: *mut void) -> c_int
    fn unsubscribe_on(channel: c_int, id: c_int)
    @ffi(link_name="tick")
    safe fn tick(channel: c_int, value: c_int) -> c_int
    @ffi(link_name="removed")
    safe fn removed() -> c_int

fn release(owned token: Option[Retained[ForeignCallback]]):
    pass

fn main():
    (none_id, none_token) = subscribe_on(1, None)
    println(none_id, none_token.is_none(), removed())
    release(none_token)
    println(removed())
    base: c_int = 100
    (id, token) = subscribe_on(2, Some(fn(value: c_int) -> c_int => value + base))
    println(id, tick(2, 5))
    release(token)
    println(removed(), tick(2, 5))
    (again, other) = subscribe_on(3, Some(fn(value: c_int) -> c_int => value * 2))
    println(again, tick(3, 4))
    release(other)
    println(removed())

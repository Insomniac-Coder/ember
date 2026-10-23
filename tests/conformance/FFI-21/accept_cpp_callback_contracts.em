#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-23
#$ profiles: debug, development, release
#$ stdout: 8
#$ stdout: -1 1 15 1
#$ stdout: 0 true
#$ stdout: 1 1 21
#$ stdout: 0
#$ stdout: -1 42
# `[FFI-21]`, `[FFI-11]` — the callback contracts of ODR-245, ODR-246 and ODR-253 on C++ functions,
# through an overlay (AB12's path: the import's wrapper and its thunk): `user_data=right@2` names
# the second of two pointers; a `nullable` callback takes an `Option` of a callable value, on a
# function and on a method, the wrapper passing it on to the thunk; and `destroys(cancel)` is given
# the id `watch` returned, which the token keeps: `watch` returns the id and an `Option` of the
# token (`None` registered nothing), and the token's end cancels that id.

from std.ffi import c_int, ForeignCallback, Retained
import cpp "hooks.hpp" with (overlay = "overlays/hooks.em")

fn main():
    base: c_int = 7
    println(cpp.pick_with(fn(left: *mut void) -> c_int => base + 1))
    missing = cpp.maybe_hook(None, 5)
    both = cpp.given_both()
    println(missing, both, cpp.maybe_hook(Some(fn(value: c_int) -> c_int => value * 3), 5), cpp.given_both())
    (nothing, no_token) = cpp.watch(None)
    println(nothing, no_token.is_none())
    (id, token) = cpp.watch(Some(fn(value: c_int) -> c_int => value + base * 2))
    println(id, cpp.watching(), cpp.poke(7))
    end(token)
    println(cpp.watching())
    panel = cpp.Panel.new()
    println(panel.get().on_resize(None, 3), panel.get().on_resize(Some(fn(size: c_int) -> c_int => size * 6), 7))

fn end(owned token: Option[Retained[ForeignCallback]]):
    pass

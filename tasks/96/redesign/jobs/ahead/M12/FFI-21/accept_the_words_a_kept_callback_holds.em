# Clause FFI-21.2: "its capture list says what it holds (`owns`, `copy of`, `thread_shared`;
# `shares`, `shares mut` and `weakref` only under `threads(creator)`"
#$ test: compile-pass
#$ rules: FFI-21, FFI-52, CLO-2
# `[FFI-52]`'s own program (its `token`, never read, is `_token` here: `[LNT-1]`, Q1), with a second
# callback that holds `copy of` a number. The declarations are never linked: the test only compiles.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(owns fn() owns game: game.add_point())

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    _token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))
    poll_events()
    println(game.score)
    n = 3
    _counted = register_cb(owns fn() copy of n: println(n))

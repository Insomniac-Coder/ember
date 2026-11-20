# Clause FFI-22.1: "A callback C may call from any thread, borrowed or kept, is a `thread_shared fn()`
# (Part VI): every value it holds is `Send` (`[THR-8]`): a one-thread owner (`shares`, `shares mut`,
# a `weakref` link of one) is `E7015`, a thread-bound value (a lock's guard, a raw pointer, a
# `threads(creator)` handle) `E7008`"; "The `E7015` message for a kept callback that holds a `shares`
# owner gives the lines that share the value between threads"
#$ test: compile-fail
#$ rules: FFI-22, FFI-21, THR-8
# `[FFI-52]`'s game with its callback under `threads(any)`: the spec's message. The raw pointer is
# code only. The declarations are never linked: the test only compiles.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
    safe fn register_cb(owns cb: thread_shared fn(), ud: *mut void)

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(owns fn() owns game: game.add_point())    #$ error[E7015]: C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `game = thread_shared Game(names=[String("ann"), String("bo")])` and `token = setup(game)` in `main`, `fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:` and `return register_cb(owns fn() thread_shared game: game.add_point())` in `setup`, and, since `add_point` changes `score`, `from std.sync import Atomic` at the top, `score: Atomic[i32] = Atomic(0)`, `this.score.fetch_add(1)` as `add_point`'s body, and `println(game.score.load())`

fn keep_pointer(p: *mut i32) -> Retained[ForeignCallback]:
    return register_cb(owns fn() copy of p: println("kept"))    #$ error[E7008]

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    _token = setup(shares mut game)
    println(game.score)

# Clause FFI-21.2: "With `callback(retained)` or `callback(once)` C keeps the function after the
# call: its capture list says what it holds (`owns`, `copy of`, `thread_shared`; `shares`, `shares
# mut` and `weakref` only under `threads(creator)`, or under `threads(main)` where the registering
# call is in a function only the main thread runs (...), since elsewhere the owner reaches another
# thread (`[THR-2]`, `E7015`"; "and a name it uses with no word is `E7004`"
#$ test: compile-fail
#$ rules: FFI-21, FFI-22, THR-2, THR-20
# The spec's two messages, with this file's line numbers. The declarations are never linked.

import std.thread
from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0

    fn add_point():
        this.score += 1

class Log:
    lines: Array[String] = []

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(unregister))
    safe fn on_main(owns cb: fn(), ud: *mut void)

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(owns fn(): game.add_point())    #$ error[E7004]: C keeps this function after `setup` returns (`callback(retained)`), so it cannot use `setup`'s `game`; say what it holds: write `return register_cb(owns fn() owns game: game.add_point())` in place of line 31 (`setup`'s owner of the game), or capture `fn() weakref game` with `if game exists as ref mut g: g.add_point()` as its body (a link that lets the game end)

fn worker():
    log = shares mut Log()
    _kept = on_main(owns fn() shares mut log: log.lines.push(owns String("w")))    #$ error[E7015]: C runs this function on the main thread (`on_main`'s `cb` says `threads(main)`), and `worker` may run on another thread (`thread.spawn`, line 40), so `log`, a `shares mut` owner of that thread, would be used by two threads; share the log between threads: write `log = thread_shared Log()` in place of line 34, with `lines` `guarded by` a lock, or register the callback from `main`

fn main():
    game = shares mut Game()
    _token = setup(shares mut game)
    h = thread.spawn(owns fn(): worker())
    h.join()

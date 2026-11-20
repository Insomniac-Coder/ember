# Clause FFI-52.1: "a call of one counts as possibly running, on the calling thread, every Ember
# function C can reach then: every callback C keeps (...) whose thread contract allows this thread";
# "and to every function that calls it: a `fn tick(): poll_events()` changes what the kept callbacks
# change, and so does every function that calls `tick`"; "A missing mark only refuses a line that
# the reachable functions' changes would make unsafe, and the message names the mark (`E3020`"
#$ test: compile-fail
#$ rules: FFI-52, FFI-21, BCK-3
# `[FFI-52]`'s program with a callback that pushes to `game.names`, as its last paragraph says:
# `sqrt` with no mark (the spec's message, with this file's line numbers; "the callback C keeps from
# `register_cb` (line N)" read as the registering call's line, Q2), `poll_events` and `tick` inside
# the walk (code only). The declarations are never linked: the test only compiles.

class Game:
    score: i32 = 0
    names: Array[String] = []

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    safe fn poll_events()
    safe fn sqrt(x: f64) -> f64

fn tick():
    poll_events()

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = register_cb(owns fn() shares mut game: game.names.push(owns String("new")))
    for name in game.names:
        d = name.len() as f64
        println(name, sqrt(d))    #$ error[E3020]: `sqrt(d)` (line 33) is a C function, which may run the callback C keeps from `register_cb` (line 30), and that callback changes `game.names`, which this loop walks (line 31); if `sqrt` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn sqrt(x: f64) -> f64`; otherwise move line 33 below the loop
        poll_events()    #$ error[E3020]
        tick()    #$ error[E3020]
    mem.drop(owns token)

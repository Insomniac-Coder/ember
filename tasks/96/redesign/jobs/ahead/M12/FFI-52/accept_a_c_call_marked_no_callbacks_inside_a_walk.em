# Clause FFI-52.2: "A foreign function that never calls back into Ember says so on its declaration or
# overlay entry, `@ffi(no_callbacks)`: a call of it runs none of them."
#$ test: compile-pass
#$ rules: FFI-52, FFI-10
# The walk of reject_a_foreign_call_may_run_what_c_keeps with `sqrt` marked, and `poll_events` and
# `tick` after the loop. The declarations are never linked: the test only compiles.

class Game:
    score: i32 = 0
    names: Array[String] = []

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn tick():
    poll_events()

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = register_cb(owns fn() shares mut game: game.names.push(owns String("new")))
    for name in game.names:
        println(name, sqrt(name.len() as f64))
    poll_events()
    tick()
    println(game.names.len())
    mem.drop(owns token)

# Clause FFI-21.3: "Handing a function to a `callback(retained)` or `callback(once)` parameter counts
# as making what it changes, now and whenever C runs it (`[FFI-52]`)."
#$ test: compile-fail
#$ rules: FFI-21, FFI-52
# Registering, inside a walk of `game.names`, a callback that pushes to it: code only. The
# declarations are never linked.

class Game:
    names: Array[String] = []

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(owns cb: fn(), ud: *mut void)

fn main():
    game = shares mut Game(names=[String("ann")])
    for name in game.names:
        println(name)
        token = register_cb(owns fn() shares mut game: game.names.push(owns String("new")))    #$ error[E3020]
        mem.drop(owns token)

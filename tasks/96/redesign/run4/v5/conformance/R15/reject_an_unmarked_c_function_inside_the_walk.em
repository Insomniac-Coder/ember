#$ test: compile-fail
#$ rules: BRW-18, CTL-2
## design rules: R15, R12, R8
## R12 (verdict 31): a C function that never calls back but is declared without the mark counts as
## possibly running the kept callback; "a missing mark only refuses such a loop, and the message names
## the mark" (AN-78; section 3's message in this program's lines; its other way out, moving the call
## below the loop, does not compile here, since the call uses the loop's `name`, so the message gives
## only the mark, R18). The nearest allowed program is
## `accept_a_c_function_marked_no_callbacks_inside_a_walk.em`.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1
        this.names.push(String("point"))

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))    #$ error[E3020]: `sqrt(name.len() as f64)` (line 34) is a C function, which may run the callback C keeps from `register_cb` (line 28), and that callback changes `game.names` (through `add_point`, line 18), which this loop walks (line 33); if `sqrt` never calls back into Ember, say so on its declaration: write `@ffi(no_callbacks)` above line 25
    poll_events()
    println(game.score)

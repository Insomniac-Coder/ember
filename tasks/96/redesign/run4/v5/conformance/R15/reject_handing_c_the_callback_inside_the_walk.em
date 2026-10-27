#$ test: compile-fail
#$ rules: R15, R12, R8
## R12 (verdicts 30 and 86): handing C a callback to keep counts as making what the callback changes, now
## or later, worked out from its body; this one adds to `game.names` (through `add_point`), so handing
## it over inside a walk of `game.names` is E3020 (AN-78; the message names the chain, R18). The nearest
## allowed program hands it over before the walk
## (`accept_a_c_function_marked_no_callbacks_inside_a_walk.em`).

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
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    for name in game.names:
        token = setup(shares mut game)    #$ error[E3020]: `setup(shares mut game)` (line 33) hands C a callback to keep (line 28), which adds to `game.names` whenever C runs it (through `add_point`, line 17), while this loop walks `game.names` (line 32); hand it over before the loop: move line 33 above line 32
        println(name)

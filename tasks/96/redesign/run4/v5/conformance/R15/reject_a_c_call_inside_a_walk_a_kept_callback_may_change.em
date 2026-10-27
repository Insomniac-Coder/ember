#$ test: compile-fail
#$ rules: BRW-18, CTL-2
## design rules: R15, R12, R8
## R15: "Every foreign call counts as possibly running the callbacks C keeps, unless the foreign
## function's declaration says `@ffi(no_callbacks)`"; R12: it "makes every change they make", worked out
## from the callback's body (verdict 86); inside a walk of what they change it is E3020 (verdict 31;
## AN-78; section 3's message in this program's lines). The nearest allowed program is
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
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))
        poll_events()    #$ error[E3020]: `poll_events()` (line 35) is a C function, which may run the callback C keeps from `register_cb` (line 28), and that callback changes `game.names` (through `add_point`, line 17), which this loop walks (line 33); call `poll_events()` before or after the loop: move line 35 below the loop; if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`
    println(game.score)

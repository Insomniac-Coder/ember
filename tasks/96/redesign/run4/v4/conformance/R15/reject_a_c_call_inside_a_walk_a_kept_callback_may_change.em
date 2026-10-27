#$ test: compile-fail
#$ rules: R15, R12, R8
## R15: "Every foreign call counts as possibly running the callbacks C keeps, unless the foreign
## function's declaration says `@ffi(no_callbacks)`"; inside a walk of what they change it is E3020
## (verdict 31; AN-78, section 3's message in this program's lines).

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    for name in game.names:
        println(name, sqrt(name.len() as f64))
        poll_events()    #$ error[E3020]: this loop walks `game.names` (line 30), and `poll_events()` (line 32) is a C function, which may run the callback C keeps from `register_cb` (line 25), and that callback changes `Game` (`fn() with owns game changes Game`), so it may change `game.names`; call `poll_events()` before or after the loop; if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`
    println(game.score)

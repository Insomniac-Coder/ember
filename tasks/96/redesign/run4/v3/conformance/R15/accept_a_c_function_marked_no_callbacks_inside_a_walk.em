#$ test: compile-pass
#$ rules: R15, R12, R5
## R12 and R15 (verdicts 30, 31): handing C a callback to keep counts as making its changes (`setup`
## and `main` say `changes Game`); a foreign function marked `@ffi(no_callbacks)` runs none of them, so
## it may be called inside a walk of `game.names`; `poll_events()`, which may, is called after it
## (AN-78's NEW).

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
    poll_events()
    println(game.score)

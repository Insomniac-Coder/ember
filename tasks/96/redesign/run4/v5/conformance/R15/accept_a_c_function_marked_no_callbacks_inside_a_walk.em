#$ test: compile-pass
#$ rules: THR-1, CLO-1, FN-4
## design rules: R15, R12, R5
## R12 and R15 (verdicts 31 and 86): handing C a callback to keep counts as making its changes, worked
## out from the callback's body, with no word on `setup`'s or `main`'s first line; a foreign function
## marked `@ffi(no_callbacks)` runs none of them, so it may be called inside a walk of `game.names`;
## `poll_events()`, which may, is called after it (AN-78's NEW, with an `add_point` that also adds a
## name, so that the kept callback changes the list the loop walks).

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
    poll_events()
    println(game.score)

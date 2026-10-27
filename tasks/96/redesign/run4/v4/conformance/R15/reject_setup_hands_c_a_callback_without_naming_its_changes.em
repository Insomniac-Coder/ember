#$ test: compile-fail
#$ rules: R15, R12, R5
## R12 (verdict 30): "a foreign function's parameter whose contract says `callback(retained)` ...
## counts" as making the function's changes, now or later; `setup` names them (AN-78, section 3's
## message).

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

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:    #$ error[E2105]: `setup` hands `register_cb` a callback that C keeps and that changes `Game`; add it to `setup`'s first line: `fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:`
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

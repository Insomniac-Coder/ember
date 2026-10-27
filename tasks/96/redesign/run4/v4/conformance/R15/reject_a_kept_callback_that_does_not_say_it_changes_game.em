#$ test: compile-fail
#$ rules: R15, R12
## R12 (verdicts 17, 29): the callback changes the game through an owner others share, so its first
## line names the type, `changes Game` (AN-78's message).

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
    return register_cb(fn() with owns game: game.add_point())    #$ error[E2105]: this function changes `game`, a `Game` other owners share, through `add_point` (`changes score`); say so on its first line: `fn() with owns game changes Game:`

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

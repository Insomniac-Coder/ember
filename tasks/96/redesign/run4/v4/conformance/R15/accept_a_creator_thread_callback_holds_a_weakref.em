#$ test: compile-pass
#$ rules: R15, R11, R12, R13
## R15: "`with shares`, `with shares mut` and `with weakref` only where the function stays on this
## thread (`threads(creator)` or `threads(main)`, checked by the trampoline)": E7004's second way out,
## a link that lets the game end, written as a named local function (verdict 51).

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
    fn point() with weakref game changes Game:
        if game exists as ref mut g:
            g.add_point()
    return register_cb(point)

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

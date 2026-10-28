#$ test: compile-fail
#$ rules: R15, R12, R5
## R5 and R12: a function that calls one whose first line changes something names it too: `main` calls
## `setup`, which says `changes Game` (AN-78).

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

fn main():    #$ error[E2105]: `main` calls `setup`, which changes `Game` (line 23: it hands C a callback that changes `Game`); add it to `main`'s first line: `fn main() changes Game:`
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

#$ test: compile-pass
#$ rules: THR-1, CLO-1, BRW-16, FN-4
## design rules: R15, R12, R8, R5
## R12 and R8 (c) (verdict 86): what the kept callback changes is worked out from its body: `add_point`
## changes `game.score` and not `game.names`, so `poll_events()`, which may run it, is allowed inside a
## walk of `game.names` (R8 (c) refuses a change of the walked list, and a number of the same object is
## not part of it). Before verdict 86 the callback's first line named the whole `Game`, which refused
## this loop. It replaces step 3's `reject_a_kept_callback_that_does_not_say_it_changes_game.em`, whose
## fault, a word missing from the callback's first line (E2105), verdict 86 removed.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

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
        println(name)
        poll_events()
    println(game.score)

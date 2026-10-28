#$ test: compile-fail
#$ rules: THR-17, THR-10, FFI-21
## design rules: R15, R12
## R15: "An unscoped thread, and a C callback that C keeps, say in the capture list what they hold
## (`owns`, `thread_shared`, `copy of`; E7004 for a name with no word)" (AN-78's message, with the lines
## to write). The nearest allowed programs are `accept_a_c_function_marked_no_callbacks_inside_a_walk.em`
## (`owns game`) and `accept_a_creator_thread_callback_holds_a_weakref.em` (`weakref game`).

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
    return register_cb(fn(): game.add_point())    #$ error[E7004]: C keeps this function after `setup` returns (`callback(retained)`), so it cannot use `setup`'s `game`; say what it holds: write `return register_cb(fn() owns game: game.add_point())` in place of line 26 (`setup`'s owner of the game, handed over), or, for a link that lets the game end, `fn point() weakref game:` with `if game exists as ref mut g:` and `g.add_point()` inside it, then `return register_cb(point)`, in place of line 26

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

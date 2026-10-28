#$ test: compile-pass
#$ rules: THR-1, WK-11, CLO-1, GRM-44
## design rules: R15, R11, R12, R13
## R15: "`shares`, `shares mut` and `weakref` only where the function stays on this thread
## (`threads(creator)` or `threads(main)`, checked by the trampoline in every profile)": E7004's second
## way out, a link that lets the game end, written as a named local function (verdict 51) whose capture
## follows its brackets with no `with` (verdict 83).

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
    fn point() weakref game:
        if game exists as ref mut g:
            g.add_point()
    return register_cb(point)

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

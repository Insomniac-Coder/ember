#$ test: compile-pass
#$ rules: BRW-18, CTL-2
## design rules: R15, R12, R5
## R12 (verdicts 39 and 86): `tick` calls `poll_events()`, which may run the kept callback that changes
## the game and pushes to `LOG`; `tick` and `main` make those changes with no word on their first lines
## (the compiler works them out, R5), and `tick()` comes after the walk of `LOG` (the line E3020 gives in
## `reject_tick_called_inside_a_walk_of_log_may_run_the_kept_callback.em`). Renamed from step 3's
## `accept_tick_names_what_the_kept_callback_changes.em`.

from std.ffi import ForeignCallback, Retained

static mut LOG: Array[String] = []

class Game:
    score: i32 = 0

    fn add_point():
        this.score += 1
        LOG.push(String("point"))

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn tick():
    poll_events()

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    for line in LOG:
        println(line)
    tick()
    println(game.score)

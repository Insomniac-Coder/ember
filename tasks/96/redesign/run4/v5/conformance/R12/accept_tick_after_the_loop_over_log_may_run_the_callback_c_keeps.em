#$ test: compile-pass
#$ rules: BRW-18, CTL-2
## design rules: R12, R5, R15
## Verdicts 30, 31, 39 and 86, R12 (rules.md lines 726-751): `setup` hands C a callback to keep, which
## changes the game and pushes to `LOG`; `tick` makes a foreign call that may run it; no first line says
## what either changes, the compiler works it out. The walk of `LOG` ends before `tick()` runs. The nearest
## allowed program to reject_tick_inside_a_loop_over_log_may_run_the_callback_c_keeps. compile-pass only:
## the C library is not part of the test.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0

    fn add_point():
        this.score += 1

static mut LOG: Array[String] = []

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    fn on_event() owns game:
        game.add_point()
        LOG.push(String("event"))
    return register_cb(on_event)

fn tick():
    poll_events()

fn main():
    LOG.push(String("start"))
    game = shares mut Game()
    token = setup(shares mut game)
    for entry in LOG:
        println(entry)
    tick()
    println(game.score, LOG.len())

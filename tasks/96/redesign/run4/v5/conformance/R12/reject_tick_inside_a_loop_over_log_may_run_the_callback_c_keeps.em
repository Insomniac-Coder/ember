#$ test: compile-fail
#$ rules: R12, R5, R8, R15
#$ help: move line 44 below the loop
## Verdicts 31, 39 and 86, R12 (rules.md lines 740-751): "a call of a foreign function counts as possibly
## running every callback C keeps ..., and makes every change they make, for R8 and for every function
## that calls it"; `tick`, which calls `poll_events()`, changes what the kept callback changes (it pushes
## to `LOG`), with no word on `tick`, so `tick()` inside a loop over `LOG` is refused (E3020). rules.md
## section 3's line 1066 `poll_events()` message, in this program's lines, naming the chain (R18). The
## callback is `threads(main)`, so it may change the main thread's `LOG` (E7012 otherwise). W6 covers the
## C side; this is R12's sentence. The nearest allowed program is
## accept_tick_after_the_loop_over_log_may_run_the_callback_c_keeps.

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
        tick()    #$ error[E3020]: `tick()` (line 44) may add to `LOG` (through `poll_events()`, line 36, a C function that may run the callback C keeps from `register_cb`, line 33, which pushes to `LOG`, line 32) while this loop walks `LOG` (line 42); call `tick()` after the loop: move line 44 below the loop; if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`
    println(game.score, LOG.len())

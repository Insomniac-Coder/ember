#$ test: compile-fail
#$ rules: R15, R12, R8
## R12 (verdicts 39 and 86): `tick` calls `poll_events()`, a C function that may run the callback C
## keeps, so `tick` "changes what the callback changes, as does every function that calls `tick`",
## worked out with no word on any first line; the callback pushes to `LOG` (through `add_point`), so a
## call of `tick()` inside a walk of `LOG` is E3020, the message naming the chain (R5, R18). It replaces
## step 3's test of a word missing from `tick`'s first line (E2105), which verdict 86 removed.
## The nearest allowed program is `accept_tick_calls_poll_events_with_no_word_on_its_first_line.em`.

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
        tick()    #$ error[E3020]: `tick()` (line 40) pushes to `LOG` (through `poll_events()`, line 33, a C function, which may run the callback C keeps from `register_cb` (line 30), which runs `add_point`, which pushes to `LOG`, line 19) while this loop walks `LOG` (line 38); call `tick()` after the loop: move line 40 below the loop
    println(game.score)

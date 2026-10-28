#$ test: run-fail
#$ rules: THR-1, CLO-1
#$ stdout: 1
#$ panics: C called the callback registered on line 29 a second time; it was registered `callback(once)`, so it may run only once
## design rules: R15, R12
## R12 and R15: a `callback(once)` function is used up by its one call; when C calls it again the
## trampoline's used-up mark stops the program before the used-up state is read (completeness G24;
## AddressSanitizer: no report). The stop's text is section 2's (the lead's, worklist section 3, F-14,
## U9), in this program's lines.

from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0

    fn add_point():
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(once, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)
    poll_events()
    println(game.score)

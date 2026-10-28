#$ test: run-fail
#$ rules: R15, R12
#$ stdout: 1
#$ panics: C called the callback registered on line 29
## R12 and R15: a `callback(once)` function is used up by its one call; when C calls it again the
## trampoline finds its slot ended and stops the program before the used-up state is read (completeness
## G24; AddressSanitizer: no report). Section 2 gives no text for this stop: the test checks the start
## its two trampoline texts share.

from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(once, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)
    poll_events()
    println(game.score)

#$ test: run-fail
#$ rules: R15
#$ profiles: debug, development, release
#$ panics: C called the callback registered on line 35 on another thread, but its contract says `threads(creator)`
## R15: a call "on a thread the callback's contract does not allow, stops the program (section 2)",
## "checked by the trampoline in every profile" (today: debug only, `tests/conformance/FFI-33b`); this
## check is what lets a kept callback hold a one-thread owner (`with owns game` of a `shares mut` game).
## The stop's text is section 2's, in this program's lines.

from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()
    @ffi(link_name="ke_poll_elsewhere")
    safe fn poll_events_elsewhere()
    @ffi(link_name="ke_fire_stale")
    safe fn fire_stale()
    @ffi(no_callbacks, link_name="ke_last_tag")
    safe fn last_tag() -> u32

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events_elsewhere()
    println(game.score)

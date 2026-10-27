#$ test: run-fail
#$ rules: R15
#$ stdout: 1
#$ panics: C called the callback registered on line 34 after its token ended; it was unregistered
## R15: "a call after the token ended ... stops the program (section 2)": the trampoline compares the
## slot's generation, so the stale call runs no freed state (AN-78); the stop's text is section 2's, in
## this program's lines.

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

fn finish(owns token: Retained[ForeignCallback]):
    pass

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)
    finish(token)
    fire_stale()
    println(game.score)

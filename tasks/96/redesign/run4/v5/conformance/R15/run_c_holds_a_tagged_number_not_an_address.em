#$ test: run-pass
#$ rules: R15
#$ stdout: 16512
#$ stdout: 1
## R15: "C holds a tagged number, never an address, with the callback's generation in its top 16 bits
## and its slot in its low 48 (AN-78's callback carries `0x4080`, 16512, in the top 16 bits)" (the
## layout is the lead's, worklist section 3, U8).

from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0

    fn add_point():
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

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    println(last_tag())
    poll_events()
    println(game.score)

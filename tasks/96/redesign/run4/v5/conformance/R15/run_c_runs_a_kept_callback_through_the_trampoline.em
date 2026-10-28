#$ test: run-pass
#$ rules: THR-1, CLO-1
#$ stdout: 2
## design rules: R15, R12
## R15: "C callbacks go through the checked trampoline"; each C call of the kept callback adds a point
## to the game it holds (section 2: the checked trampoline, 1.02x to 1.05x of plain C, with its thread
## check in every profile; AN-78).

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
    poll_events()
    poll_events()
    println(game.score)

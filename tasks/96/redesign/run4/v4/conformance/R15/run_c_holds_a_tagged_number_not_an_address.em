#$ test: run-pass
#$ rules: R15
#$ stdout: 16512
#$ stdout: 1
## R15: "C holds a tagged `{slot, generation}` number, never an address"; AN-78: "the top 16 bits are
## `0x4080`, not an address" (16512). The tag's value is AN-78's; `V4/rules.md` gives no layout (open,
## see the matrix, R15-17).

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
    println(last_tag())
    poll_events()
    println(game.score)

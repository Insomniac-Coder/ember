#$ test: run-pass
#$ rules: R15, R12
#$ stdout: 1
## R12 and R15 (verdict 62): for a C callback that ends its own `Retained` token while it runs, "the
## token's end unregisters the callback at once, so C never runs it again, but the block that holds its
## state ... is ended and freed only when the callback returns, so the rest of its body runs as
## written": here `me.add_point()` after `me.token = None` (completeness G27; AddressSanitizer: no
## report). The callback is a named local function (verdict 51).

from std.ffi import ForeignCallback, Retained

import c "./kept_events.c" as raw

class Game:
    score: i32 = 0
    token: Option[Retained[ForeignCallback]] = None

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    @ffi(link_name="ke_unregister")
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister), link_name="ke_register")
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(link_name="ke_poll")
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game:
    me = shares mut game
    fn leave() with owns me changes Game:
        me.token = None
        me.add_point()
    game.token = Some(register_cb(leave))

fn main() changes Game:
    game = shares mut Game()
    setup(shares mut game)
    poll_events()
    poll_events()
    println(game.score)

#$ test: compile-fail
#$ rules: FFI-52
## design rules: R15, R12
## R12: "`@ffi(no_callbacks)` ... on a function whose own contract takes a callback it is refused
## (E5069)" (AN-78, section 3's message).

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(no_callbacks, param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))    #$ error[E5069]: `register_cb` takes a callback (`param(cb, callback(retained, user_data=ud))`), so it may call back into Ember; remove `no_callbacks` from its `@ffi(...)`
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())

fn main():
    game = shares mut Game()
    token = setup(shares mut game)
    println(game.score)

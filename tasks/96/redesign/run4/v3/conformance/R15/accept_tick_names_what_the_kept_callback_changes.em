#$ test: compile-pass
#$ rules: R15, R12, R5
## R12 (verdict 39): `fn tick() changes Game, LOG: poll_events()`, and `main`, which calls `tick`,
## names them too.

from std.ffi import ForeignCallback, Retained

static mut LOG: Array[String] = []

class Game:
    score: i32 = 0

    fn add_point() changes score, LOG:
        this.score += 1
        LOG.push(String("point"))

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) changes Game, LOG -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game, LOG: game.add_point())

fn tick() changes Game, LOG:
    poll_events()

fn main() changes Game, LOG:
    game = shares mut Game()
    token = setup(shares mut game)
    tick()
    println(game.score)

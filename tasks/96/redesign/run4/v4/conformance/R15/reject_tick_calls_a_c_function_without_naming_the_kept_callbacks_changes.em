#$ test: compile-fail
#$ rules: R15, R12, R5
## R12 (verdict 39): "A function that makes such a call names the kept callbacks' changes on its first
## line, as it names what any function it calls changes, and so does every function that calls it"
## (R12's own message).

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

fn tick():    #$ error[E2105]: `tick` calls `poll_events()`, a C function that may run the callback `setup` gave C, which changes `Game` and `LOG`; write `fn tick() changes Game, LOG:`, or, if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`
    poll_events()

fn main() changes Game, LOG:
    game = shares mut Game()
    token = setup(shares mut game)
    tick()
    println(game.score)

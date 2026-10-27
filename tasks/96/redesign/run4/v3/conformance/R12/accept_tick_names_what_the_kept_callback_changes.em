#$ test: compile-pass
#$ rules: R12, R5, R15
## Verdicts 30, 31 and 39, R12 (rules.md lines 570-578, 589-602): `setup` hands C a callback to keep and
## names what it changes; `tick` makes a foreign call that may run it and names the same; `main` calls
## both. compile-pass only: the C library is not part of the test.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0

    fn add_point() changes score:
        this.score += 1

static mut LOG: Array[String] = []

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game, LOG -> Retained[ForeignCallback]:
    fn on_event() with owns game changes Game, LOG:
        game.add_point()
        LOG.push(String("event"))
    return register_cb(on_event)

fn tick() changes Game, LOG:
    poll_events()

fn main() changes Game, LOG:
    game = shares mut Game()
    token = setup(shares mut game)
    tick()
    println(game.score, LOG.len())

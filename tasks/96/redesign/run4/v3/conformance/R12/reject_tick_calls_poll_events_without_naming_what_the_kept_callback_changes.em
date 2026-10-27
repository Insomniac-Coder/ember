#$ test: compile-fail
#$ rules: R12, R5, R15
#$ help: fn tick() changes Game, LOG:
## Verdict 39, R12 (rules.md lines 589-602): a foreign call counts as possibly running every callback C
## keeps, so a function that makes one names what they change, as every function that calls it does.
## rules.md lines 597-601's message, word for word. The callback is `threads(main)`, so it may change the
## main thread's `LOG` (E7012 otherwise, AN-76 rule 4). W6 covers the C side; this is R12's sentence.

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

fn tick():    #$ error[E2105]: `tick` calls `poll_events()`, a C function that may run the callback `setup` gave C, which changes `Game` and `LOG`; write `fn tick() changes Game, LOG:`, or, if `poll_events` never calls back into Ember, say so on its declaration: `@ffi(no_callbacks) safe fn poll_events()`
    poll_events()

fn main() changes Game, LOG:
    game = shares mut Game()
    token = setup(shares mut game)
    tick()
    println(game.score, LOG.len())

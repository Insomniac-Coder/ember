#$ test: compile-fail
#$ rules: R15, R12
## R15: "An unscoped thread, and a C callback that C keeps, say in the capture list what they hold ...
## E7004 for a name with no word" (AN-78's message).

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()
    @ffi(no_callbacks)
    safe fn sqrt(x: f64) -> f64

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() changes Game: game.add_point())    #$ error[E7004]: C keeps this function after `setup` returns (`callback(retained)`), so it cannot use `setup`'s `game`; say what it holds: `fn() with owns game changes Game:` (`setup`'s owner of the game), or `fn() with weakref game changes Game:` with `if game exists as ref mut g: g.add_point()` as its body (a link that lets the game end)

fn main() changes Game:
    game = shares mut Game()
    token = setup(shares mut game)
    poll_events()
    println(game.score)

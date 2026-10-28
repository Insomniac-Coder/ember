#$ test: compile-fail
#$ rules: R15, R12, R8
## R12 (verdict 30): handing C a callback that changes `Game` is a change of `Game`, so inside a walk
## of `game.names` it is E3020 (AN-78's message, in this program's lines).

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
    return register_cb(fn() with owns game changes Game: game.add_point())

fn main() changes Game:
    game = shares mut Game(names=[String("ann"), String("bo")])
    for name in game.names:
        token = setup(shares mut game)    #$ error[E3020]: this loop walks `game.names` (line 28), and `setup(shares mut game)` (line 29) changes `Game` (its first line says `changes Game`: it hands C a callback that changes `Game`); call it before the loop: write `token = setup(shares mut game)` above the `for` line
        println(name)

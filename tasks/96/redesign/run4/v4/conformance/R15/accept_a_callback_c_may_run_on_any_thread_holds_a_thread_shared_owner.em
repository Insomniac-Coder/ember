#$ test: compile-pass
#$ rules: R15, R9, R12, R5
## R15: the lines E7015 gives: a `threads(any)` callback holds a `thread_shared` owner, and the score
## it changes is an `Atomic`, changed by its own operation (R5; verdict 29 names it, `changes score`).

from std.ffi import ForeignCallback, Retained
from std.sync import Atomic

class Game:
    score: Atomic[i32] = Atomic(0)
    names: Array[String] = []

    fn add_point() changes score:
        this.score.fetch_add(1)

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(game: thread_shared Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with thread_shared game changes Game: game.add_point())

fn main() changes Game:
    game = thread_shared Game(names=[String("ann"), String("bo")])
    token = setup(game)
    poll_events()
    println(game.score.load())

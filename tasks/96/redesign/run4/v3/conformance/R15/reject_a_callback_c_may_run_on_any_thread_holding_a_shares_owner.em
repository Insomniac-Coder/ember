#$ test: compile-fail
#$ rules: R15, R9
## R15: "a one-thread owner ... reaching another thread is E7015": a `threads(any)` callback may not
## hold a `shares mut` owner (AN-78, section 3's message).

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point() changes score:
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game: game.add_point())    #$ error[E7015]: C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `game = thread_shared Game(names=[String("ann"), String("bo")])` and `token = setup(game)` in `main`, `fn setup(game: thread_shared Game) changes Game -> Retained[ForeignCallback]:` and `return register_cb(fn() with thread_shared game changes Game: game.add_point())` in `setup`, and, since `add_point` changes `score`, `score: Atomic[i32] = Atomic(0)`, `this.score.fetch_add(1)` as `add_point`'s body, and `println(game.score.load())`

fn main() changes Game:
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    poll_events()
    println(game.score)

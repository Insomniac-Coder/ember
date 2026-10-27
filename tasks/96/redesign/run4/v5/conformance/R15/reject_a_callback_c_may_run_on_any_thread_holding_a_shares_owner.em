#$ test: compile-fail
#$ rules: R15, R9, R12
## R15: "`shares`, `shares mut` and `weakref` only where the function stays on this thread"; R12: a
## function C may run on any thread holds no one-thread value (E7015): a `threads(any)` callback may not
## hold a `shares mut` owner (AN-78; section 3's E7015 message, with the line each part replaces). The
## nearest allowed program is `accept_a_callback_c_may_run_on_any_thread_holds_a_thread_shared_owner.em`.

from std.ffi import ForeignCallback, Retained

class Game:
    score: i32 = 0
    names: Array[String] = []

    fn add_point():
        this.score += 1

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(owns game: shares mut Game) -> Retained[ForeignCallback]:
    return register_cb(fn() owns game: game.add_point())    #$ error[E7015]: C may call this function from any thread (`threads(any)`), and `shares` owners stay on one thread; share the game between threads: write `from std.sync import Atomic` below line 8, `score: Atomic[i32] = Atomic(0)` in place of line 11 and, since `add_point` changes `score`, `this.score.fetch_add(1)` in place of line 15; `fn setup(game: thread_shared Game) -> Retained[ForeignCallback]:` in place of line 23 and `return register_cb(fn() thread_shared game: game.add_point())` in place of line 24; and in `main` `game = thread_shared Game(names=[String("ann"), String("bo")])` in place of line 27, `token = setup(game)` in place of line 28 and `println(game.score.load())` in place of line 30

fn main():
    game = shares mut Game(names=[String("ann"), String("bo")])
    token = setup(shares mut game)
    poll_events()
    println(game.score)

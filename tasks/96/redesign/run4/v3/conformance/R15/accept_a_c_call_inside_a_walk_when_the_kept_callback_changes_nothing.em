#$ test: compile-pass
#$ rules: R15, R12, R8
## R12: "With no kept callback, or one that changes nothing, a foreign call changes nothing" (verdict
## 31: "with a kept callback that changes nothing (`fn(): println("event")`) it is allowed").

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

fn setup() -> Retained[ForeignCallback]:
    return register_cb(fn(): println("event"))

fn main():
    game = Game(names=[String("ann"), String("bo")])
    token = setup()
    for name in game.names:
        println(name)
        poll_events()

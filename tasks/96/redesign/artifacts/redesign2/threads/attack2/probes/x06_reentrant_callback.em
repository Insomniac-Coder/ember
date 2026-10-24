# A callback C runs on the main thread during a later C call (an event pump), and it grows the
# list the caller is walking. Today this passes `ember check`; what stops it is the run-time access
# check on class objects ([EXC-1]), which the proposed model removes.
from std.ffi import Retained

class Game:
    log: Array[String] = []

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(unregister))
    safe fn on_event(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn main():
    game = Game()
    game.log.push("start".to_string())
    token = on_event(owned fn(): game.log.push("event".to_string()))
    for s in game.log:
        poll_events()
        println(s)
    println(game.log.len())

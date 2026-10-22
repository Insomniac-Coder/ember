#$ test: run-pass
#$ rules: FFI-21, FFI-11, FFI-23
#$ profiles: debug, development, release
#$ stdout: queued 0
#$ stdout: done first 6
#$ stdout: done second 10
#$ stdout: ran 2
#$ stdout: 1 2
#$ stdout: 30 1
#$ stdout: 0 0
#$ stdout: 15 1
#$ stdout: 0
# `[FFI-21]`, `[FFI-11]` — callbacks C keeps past the call, through an overlay on the header:
# `callback(once, user_data=data)` takes the callable over (an `owned fn`, which moves what it
# captures into it): it lives in an allocation of its own until C calls it, once, and then ends
# with it. `callback(retained, user_data=data)` returns a `Retained` token (`[FFI-23]`) holding
# the callable; the token's end unregisters it through the declared `destroys(unlisten)`, called
# with what the registering call passed (the trampoline, the user data, the `kind`), and then
# ends the callable. Strings moved into the callables show each is the one C calls. No `unsafe`.

from std.ffi import c_int, ForeignCallback, Retained

import c "./events.c" with (overlay = "overlays/events.em") as ev

fn queue():
    first = "first".to_string()
    second = "second".to_string()
    ev.run_later(3, owned fn(result: c_int): println("done", first, result))
    ev.run_later(5, owned fn(result: c_int): println("done", second, result))

fn main():
    queue()
    println("queued", 0)
    println("ran", ev.run_pending())
    scale: c_int = 10
    tag = "a".to_string()
    token = ev.listen(1, owned fn(event: c_int) -> c_int => event * scale + tag.len() as c_int - 1)
    other = ev.listen(2, owned fn(event: c_int) -> c_int => event)
    println(ev.listening() - 1, ev.listening())
    println(ev.emit(1, 3), ev.emit(2, 1))
    drop_both(token, other)
    println(ev.emit(1, 3), ev.listening())
    again = ev.listen(1, owned fn(event: c_int) -> c_int => event * 5)
    println(ev.emit(1, 3), ev.listening())
    keep(again)
    println(ev.listening())

fn drop_both(owned a: Retained[ForeignCallback], owned b: Retained[ForeignCallback]):
    pass

fn keep(owned token: Retained[ForeignCallback]):
    pass

#$ test: parse-pass
#$ rules: FFI-23, FFI-11, FFI-22
# `[FFI-23]` — `Retained.pin(v)` hands an Ember object to C for keeping: for a
# class handle the token holds a strong count of the object (it outlives the
# handle it was pinned from) and pins its header; for a `Box` the token owns
# it. `tok.ptr()` is the address C keeps, and dropping the token releases the
# object. A parameter contracted `retained` gives C that address
# (`[FFI-11]`); a class that is not `Send` crosses only with
# `threads(main)` or `threads(creator)` (`[FFI-22]`). The C side is
# `ember_driver/tests/fixtures/foreign_objects.c`; the milestone
# `retained_tokens_let_c_keep_ember_objects` links and runs it.

from std.ffi import Retained
from std.mem import drop

class Noisy:
    name: str

    fn drop(mut self):
        println("dropped", self.name)

struct Point:
    x: i64
    y: i64

unsafe extern "C":
    @ffi(param(object, retained, one, threads(main)))
    safe fn keep(object: Retained[Noisy])
    @ffi(param(point, retained, one), link_name="keep")
    safe fn keep_point(point: Retained[Box[Point]])
    @ffi(effects=[])
    safe fn kept_point_sum() -> i64
    fn kept_is(address: *mut u8) -> i32

fn pinned() -> Retained[Noisy]:
    noisy = Noisy(name = "kept")
    token = Retained.pin(noisy)
    keep(token)
    return token

fn main():
    token = pinned()
    println("still alive")
    same: i32 = 0
    unsafe:  # SAFETY(ffi): `kept_is` compares two addresses
        same = kept_is(token.ptr())
    println(same)
    drop(token)
    println("released")
    point = Retained.pin(Box(Point(x = 3, y = 4)))
    keep_point(point)
    println(kept_point_sum())

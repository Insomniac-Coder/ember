#$ test: run-pass
#$ rules: HR-20, FFI-23
#$ profiles: debug, development, release
#$ stdout: true false
#$ 0
#$ dropped b
#$ dropped a
## `[HR-20]` (ODR-194) — `Retained.pin(handle, on_relocate: Option[extern "C" fn(ObjectPtr[C],
## ObjectPtr[C])])`: given a callback, the token's object is not pinned, and a reload that moves it
## calls the callback with the old and the new address (`ember_driver/tests/reload.rs`,
## `a_pinned_object_with_on_relocate_moves_and_is_told`); with nothing reloaded it is never called.
## The token's end releases the object as any token's does. `ObjectPtr`s are equal when they are
## the address of one object (`[FFI-23]`; that `==` was `E2040`).

from std.ffi import Retained, ObjectPtr
from std.sync import Atomic

class Noisy:
    name: str

    fn drop(mut self):
        println("dropped", self.name)

static CALLS: Atomic[i64] = Atomic(0)

extern "C" fn moved(old: ObjectPtr[Noisy], new: ObjectPtr[Noisy]):
    CALLS.fetch_add(1)

fn main():
    a = Retained.pin(Noisy(name = "a"), on_relocate=Some(moved))
    b = Retained.pin(Noisy(name = "b"))
    println(a.ptr() == a.ptr(), a.ptr() == b.ptr())
    println(CALLS.load())

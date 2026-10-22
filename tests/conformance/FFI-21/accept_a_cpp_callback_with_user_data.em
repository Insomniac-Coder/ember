#$ test: run-pass
#$ rules: FFI-21, FFI-11
#$ profiles: debug, development, release
#$ stdout: 46
#$ stdout: 7 2
#$ stdout: 6 1
#$ stdout: 0 0
# `[FFI-21]`, `[FFI-11]` — a C++ function that calls back through a function pointer and the user
# data beside it takes a callable as a C function does: the overlay's `callback(…, user_data=q)`
# makes the parameter the callable a caller passes (a lambda, a local function writing the caller's
# locals), and `q` leaves the signature (the thunk passes C++ the trampoline and the callable's
# address). A `retained` one returns the `Retained` token whose end unregisters it through the
# entry's `destroys(unlisten)`, the C++ function's declaration.

from std.ffi import c_int, ForeignCallback, Retained
import cpp "listeners.hpp" with (overlay = "overlays/listeners.em")

fn main():
    offset: c_int = 10
    println(cpp.sum_with(fn(i: c_int) -> c_int => i + offset, 4))
    bus = cpp.Bus.new()
    bus.get_mut().add(3)
    bus.get_mut().add(4)
    total: c_int = 0
    seen = 0
    fn tally(value: c_int):
        total += value
        seen += 1
    bus.get().each(tally)
    println(total, seen)
    scale: c_int = 3
    token = cpp.listen(owned fn(event: c_int) -> c_int => event * scale)
    println(cpp.emit(2), cpp.listening())
    end(token)
    println(cpp.emit(2), cpp.listening())

fn end(owned token: Retained[ForeignCallback]):
    pass

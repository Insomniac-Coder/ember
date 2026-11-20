# Clause FFI-39d.1: "call `cpp.dispatch_all()` after the loop: move line 14 below the loop"
#$ test: compile-pass
#$ rules: FFI-39d, FFI-52
# The fix the message gives. The test only compiles.

from std.ffi import c_int
import cpp "dispatch.hpp" with (overlay = "overlays/dispatch.em")

class Button(cpp.Widget):
    codes: Array[c_int] = []

    override fn on_event(code: c_int):
        this.codes.push(owns code)

fn main():
    first = Button()
    cpp.hold(owns first)
    b = Button()
    for code in b.codes:
        println(code)
    cpp.dispatch_all()

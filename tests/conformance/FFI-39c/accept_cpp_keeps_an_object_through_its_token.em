#$ test: run-pass
#$ rules: FFI-39c, FFI-23
#$ profiles: debug, development, release
#$ stdout: 7 7
#$ stdout: 7
#$ stdout: dropped
#$ stdout: -1
# `[FFI-39c]` (ODR-156) — passing an object to a C++ parameter that keeps it (`retained`) takes a
# `Retained` token the program pins and keeps (`[FFI-23]`): `Retained.pin(button)` holds a count
# of it, a token of a `Button` is one of its C++ base `Widget`, and C++ is given the address of the
# C++ object. The token keeps the object alive after its last handle ends, until the program drops
# it. A parameter that only borrows the object for the call (`draw_once`) takes the object itself.

from std.ffi import Retained
from std.mem import drop

import cpp "widgets.hpp" with (overlay = "overlays/widgets.em")

class Button(cpp.Widget):
    fn drop(mut self):
        println("dropped")

    override fn draw(self) -> i32:
        return 7

fn main():
    button = Button()
    token = Retained.pin(button)
    cpp.add_widget(token)
    println(cpp.draw_added(), cpp.draw_once(button))
    drop(button)
    println(cpp.draw_added())
    cpp.remove_widget()
    drop(token)
    println(cpp.draw_added())

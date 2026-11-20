# Clause FFI-39d.1: "a call into C++ not marked `@ffi(no_callbacks)` counts as possibly running every
# override of an Ember class derived from a C++ class whose objects C++ may hold (`[FFI-52]`), and
# makes every change those overrides make. So a walk (...) the caller holds across the call is
# refused where an override may change what it views (`E3020`"; "Nothing is checked while the
# program runs."
#$ test: compile-fail
#$ rules: FFI-39d, FFI-52, FFI-39c
# The spec's message, with this file's line numbers ("whose objects C++ may hold (line N)" read as
# the class's line, Q4). C++ holds one `Button` (`hold`, `owns`); the walk is of another's `codes`,
# which the override changes on any `Button`. The test only compiles.

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
        cpp.dispatch_all()    #$ error[E3020]: `cpp.dispatch_all()` (line 26) may run `on_event`, an override of `Button` whose objects C++ may hold (line 15), which pushes to a `Button`'s `codes`, and this loop walks `b.codes` (line 25); call `cpp.dispatch_all()` after the loop: move line 26 below the loop
        println(code)

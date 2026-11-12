#$ test: run-pass
#$ rules: FFI-39c, FFI-17d, FFI-23
#$ profiles: debug, development, release
#$ stdout: 7
#$ stdout: 7
#$ stdout: -1
# `[FFI-39c]` — a C++ parameter that keeps the pointer it is given (contracted `owns` in the
# overlay) takes the object over: its class is declared `@ffi(trampoline, owner="foreign")`
# (`[FFI-17d]`), the argument is handed over and C++ is given the address of its C++ base. A
# parameter borrowed for the call alone takes the object as any plain parameter (`draw_once`).
# `[FFI-23]` — no `Retained` token is involved: `Retained` keeps no object for C.

import cpp "widgets.hpp" with (overlay = "overlays/widgets.em")

class Button(cpp.Widget):
    override fn draw() -> i32:
        return 7

fn main():
    button = Button()
    println(cpp.draw_once(button))
    cpp.add_widget(button)
    println(cpp.draw_added())
    cpp.remove_widget()
    println(cpp.draw_added())

#$ test: run-pass
#$ rules: FFI-39d
#$ profiles: debug, development, release
#$ stdout: 2 8
# `[FFI-39d]` — re-entrant calls from C++ into the same Ember object are expected: `press` passes
# the object to C++, which calls its `on_event` back, and the override writes to it. `press` holds
# no long-term access, so nothing conflicts.

import cpp "widgets.hpp" with (overlay = "overlays/widgets.em")

class Button(cpp.Widget):
    codes: Array[i32] = []

    override fn on_event(self, code: i32):
        self.codes.push(code)

    fn press(self):
        cpp.fire(self, 4)

fn main():
    button = Button()
    button.press()
    button.press()
    println(button.codes.len(), button.codes[0] + button.codes[1])

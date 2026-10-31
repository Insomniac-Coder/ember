#$ test: run-fail
#$ rules: FFI-39d, EXC-7
#$ panics: exclusivity violation
# `[FFI-39d]` — an access the Ember caller holds across a call into C++ stays active and checked:
# `press` changes its object (`this.codes.push(0)`), a write access to every field for the call
# (`[EXC-15]`), and the override C++ calls back begins a conflicting one, which panics (`[EXC-1]`) rather than change
# what the caller holds. `L3013` warns of the long-term access held across the call (`[EXC-7]`).

import cpp "widgets.hpp" with (overlay = "overlays/widgets.em")

class Button(cpp.Widget):
    codes: Array[i32] = []

    override fn on_event(code: i32):
        this.codes.push(code)

    fn press():
        this.codes.push(0)
        cpp.fire(this, 4)    #$ warning[L3013]: long-term access to `this` is live across this call

fn main():
    button = Button()
    button.press()
    println(button.codes.len())

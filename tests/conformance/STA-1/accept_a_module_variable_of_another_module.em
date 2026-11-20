#$ test: run-pass
#$ rules: STA-1, STA-3, BRW-16
#$ stdout: 3 2 12 2
#$ stdout: a
# `[STA-1]` — another module's `pub static mut` is changed by its name too: through its module
# (`state.FRAME += 2`) or imported (`COUNT = …`), and a position of one of its elements survives a
# push (`[BRW-16]`). A `static` placed in the image is read through its module the same way.

import module_support.state as state
from module_support.state import COUNT, bump

fn main():
    bump()
    state.FRAME += 2
    state.NAMES.push(owns String("b"))
    COUNT = COUNT + 10
    println(state.FRAME, state.NAMES.len(), COUNT, state.TABLE[1])
    v = ref mut state.NAMES[0]
    state.NAMES.push(owns String("c"))
    println(v)

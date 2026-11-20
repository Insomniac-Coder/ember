# Clause FFI-33.1 with `[THR-20]`: "a function that may run on another thread (reachable from (...)
# an `@unsafe(export("name", threads=any))` or `@unsafe(export("name", threads=creator))` function)
# and reads or changes it is `E7012`"
#$ test: compile-fail
#$ rules: FFI-33, THR-20, STA-1
#$ error[E7012]
# No text in the spec for this program: code only, claimed for the file (Q3).

static mut FRAME: int = 0

# SAFETY: nothing else in the process defines this name
@unsafe(export("frame_tick", threads=any))
pub fn tick() -> int:
    FRAME += 1
    return FRAME

fn main():
    println(tick())

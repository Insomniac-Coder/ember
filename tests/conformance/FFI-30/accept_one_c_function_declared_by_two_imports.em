#$ test: run-pass
#$ rules: FFI-30
#$ warning[L3015]: `answer_value` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 42 42
# `[FFI-30]` — two imports of one header, with different defines, each declare its function: one C
# function, one callable, whichever import's name calls it.

import c "answer.h" with (implementation = ["ANSWER_IMPLEMENTATION"])
import c "answer.h" with (defines = ["ANSWER_OTHER"]) as other

fn main():
    unsafe:  # SAFETY(ffi): calls with no arguments
        println(c.answer_value(), other.answer_value())

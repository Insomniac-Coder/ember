#$ test: run-pass
#$ rules: FFI-17, FFI-24, FFI-32
#$ profiles: debug, development, release
#$ stdout: 3 4 out of order
# `[FFI-17]` — a C++ parameter may be named as the import names what it passes a thunk itself
# (`result`, `failed`): `bounds` may throw (`[FFI-24]`, where its thunk says it caught) and returns
# a class by value (`[FFI-32]`, written where the caller says), and its own `result` and `failed`
# are its parameters still.

import cpp "texts.hpp"

fn main():
    unsafe:  # SAFETY(ffi): scalar arguments
        match cpp.bounds(3, 4):
            Ok(ends):
                match cpp.bounds(4, 3):
                    Ok(_):
                        println(ends.first, ends.last, "ordered")
                    Err(error):
                        println(ends.first, ends.last, error.what)
            Err(error):
                println(error.what)

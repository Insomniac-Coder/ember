#$ test: run-pass
#$ rules: FFI-17, FFI-11, FFI-16
#$ profiles: debug, development, release
#$ stdout: 21 odd
#$ stdout: 4 40 49
# `[FFI-11]`, `[FFI-16]` (D-808) — on a method, an `out` pointer leaves the signature and the call
# returns what C++ wrote there, after the method's own result (`[FFI-11b]`); a status result is a
# `Result` of the outputs; a method that may throw gives that inside its own `Result`
# (`[FFI-24]`).

import cpp "statuses.hpp" with (overlay = "overlays/statuses.em")

fn main():
    splitter = cpp.Splitter.new()
    match splitter.get().split(42):
        Ok(half):
            match splitter.get().split(21):
                Ok(_):
                    println(half, "even")
                Err(_):
                    println(half, "odd")
        Err(_):
            println("odd")
    match splitter.get().decade(43):
        Ok((decade, low, high)):
            println(decade, low, high)
        Err(error):
            println(error.what)

#$ test: compile-fail
#$ rules: TYP-20, TYP-19, DIA-14
# D-411 — `[TYP-20]`: two implementations of one interface for one type are
# `E2041`, naming both. A program's that collides with the standard
# library's is the one the error points at, with std's under its own file's
# header; the error pointed only into `std/src/core.em`, and the program's
# file and line appeared nowhere in the rendered text.

extend int implements Default:    #$ error[E2041]: `i64` already implements `Default`
    fn default() -> int:
        return 7

fn main():
    println(int.default())

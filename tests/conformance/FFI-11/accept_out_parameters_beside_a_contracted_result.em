#$ test: run-pass
#$ rules: FFI-11, FFI-11b, FFI-36a
#$ profiles: debug, development, release
#$ stdout: 50 1
#$ stdout: 0
#$ stdout: none -1 0
#$ stdout: some 30 1
#$ stdout: 0
# `[FFI-11]`, `[FFI-11b]` — where C's own result has a contract, the call returns it first as
# what the contract makes it, then the `out` parameters: here a new object C hands over
# (`returns_owned`), boxed and freed by its destructor when the box ends, and `None` where C
# returned null (`nullable`). No `unsafe`.

import c "./tokens.c" with (overlay = "overlays/tokens.em") as t

fn made():
    _token, id = t.token_make(5)
    println(id, t.token_live())

fn main():
    made()
    println(t.token_live())
    maybe, other = t.token_maybe(-1)
    match maybe:
        Some(_):
            println("some", other, t.token_live())
        None:
            println("none", other, t.token_live())
    match t.token_maybe(3):
        (Some(_), id):
            println("some", id, t.token_live())
        (None, id):
            println("none", id, t.token_live())
    println(t.token_live())

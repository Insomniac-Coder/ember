#$ test: compile-fail
#$ rules: FFI-8, FFI-12
#$ profiles: debug
# `[FFI-8]` (ODR-249), `[FFI-12]` — an overlay's `@ffi(newtype) type T` entry names a `typedef` of the
# header, and says `newtype` alone: one naming a struct (`struct widget` has no typedef of that
# name) or with another word beside it does not match the header (`E5010`).

import c "./newtypes.c" with (overlay = "overlays/newtypes_misused.em") as nt

fn main():
    println(nt.twice(2))
#$ error[E5010]: declares no `typedef` `widget` that `@ffi(newtype)` could make nominal
#$ error[E5010]: is no argument of a newtype entry: it takes `newtype` alone

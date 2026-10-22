#$ test: compile-fail
#$ rules: FFI-8
# `[FFI-8]` — a `VaList` is never constructed (`E5018`): C makes one for the variadic call whose
# arguments it reads, and Ember only receives it, as a parameter of a function C calls.

from std.ffi import VaList
import std.ffi

fn main():
    made = VaList()    #$ error[E5018]
    qualified = ffi.VaList()    #$ error[E5018]

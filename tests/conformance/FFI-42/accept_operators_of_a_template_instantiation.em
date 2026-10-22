#$ test: run-pass
#$ rules: FFI-42, FFI-17b
#$ profiles: debug, development, release
#$ stdout: 11 22 33
#$ stdout: false true
# `[FFI-42]`, `[FFI-17b]` — the operators of a class template's instantiation map to Ember's
# operator interfaces as a class's do: `Pair<int>`'s `operator+` is `Add`, its `operator==` `Eq`,
# each taken by its address in the instantiation, as its other methods are.

import cpp "pairs.hpp" with (instantiate = ["Pair<int>"], overlay = "overlays/pairs.em")

fn main():
    a = cpp.Pair_int(first = 1, second = 2)
    b = cpp.Pair_int(first = 10, second = 20)
    c = a + b
    println(c.first, c.second, c.sum())
    println(a == b, c == cpp.Pair_int(first = 11, second = 22))

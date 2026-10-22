#$ test: run-pass
#$ rules: FFI-17a
#$ profiles: debug, development, release
#$ stdout: 3.0
#$ stdout: boxy 2.0
#$ stdout: fraction 3.5
#$ stdout: whole 4
#$ stdout: 1 0
# `[FFI-17a]` (ODR-198) — a `std::variant<…>` the header names with an alias crosses as an enum
# when every alternative maps by value and the overlay names each (`@ffi(variant=[…])` on the
# alias's `type` entry): one variant per alternative, in order, holding it. As a parameter the
# enum's alternative is made the C++ variant's for the call; a result's is read back.

import cpp "variants.hpp" with (overlay = "overlays/variants.em")

fn shown(number: cpp.Number) -> String:
    match number:
        cpp.Number.Whole(n):
            return f"whole {n}"
        cpp.Number.Fraction(x):
            return f"fraction {x}"

fn main():
    round = cpp.Shape.Round(cpp.Circle(radius = 1.0))
    println(cpp.area(round))
    match cpp.bigger(round):
        cpp.Shape.Boxy(square):
            println("boxy", square.side)
        cpp.Shape.Round(circle):
            println("round", circle.radius)
    println(shown(cpp.half(7)))
    println(shown(cpp.half(8)))
    println(cpp.kind_of(cpp.Number.Fraction(1.5)), cpp.kind_of(cpp.Number.Whole(2)))

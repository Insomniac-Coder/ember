#$ test: compile-fail
#$ rules: DIA-12, TYP-20, TYP-6
#$ help: convert it: `as f32`
#$ help: derive it, `@derive(Hash)` on `Pos`
# `[DIA-12]` (D-822) — shape N9 names the bound to add and where (`@derive(Hash)` on the program's
# own struct); shape N4 names the `as` that converts the value, and what it does to it.

from std.math import Vec3

struct Pos:
    x: int
    y: int

fn step(dx: float) -> Vec3:
    return Vec3(dx, 0.0, 0.0)    #$ error[E2020]: expected `f32`, found `f64`

fn main():
    seen: Map[Pos, int] = {}    #$ error[E2040]: `Pos` does not implement `std.collections.Hash`
    println(seen.len(), step(1.5).x)

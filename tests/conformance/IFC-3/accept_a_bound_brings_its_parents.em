#$ test: run-pass
#$ rules: IFC-3, TYP-17
#$ profiles: debug, release, shipping
#$ stdout: 2.5 [1.5, 3.5]
# ODR-042 — a bound brings its parents: `C: IndexMut[int]` reads through
# `Index[int]` too, and `Output = f32` names the parent's `Output`.

fn swap_first[C: IndexMut[int, Output = f32]](mut c: C) -> f32:
    old = c[1]
    c[1] = c[1] + 1.0
    return old

fn main():
    xs: Array[f32] = [1.5, 2.5]
    println(swap_first(xs), xs)

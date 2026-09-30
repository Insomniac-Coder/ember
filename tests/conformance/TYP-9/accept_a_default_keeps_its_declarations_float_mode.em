#$ test: run-pass
#$ rules: TYP-9, CG-C-11, FN-5
#$ profiles: debug, release, shipping
#$ assert-c-order: "1.0e-3" then "its `fastmath` functions"
#$ assert-c-order: "4.375e-1" then "its `fastmath` functions"
#$ assert-c-order: "6.25e-2" then "its `fastmath` functions"
#$ assert-c-order: "1.5625e-1" then "its `fastmath` functions"
#$ assert-c-order: "its `fastmath` functions" then "3.75e-1"
#$ stdout: 1.002 0.875 1.125 1.75 1.0 0.3125
#$ stdout: 1.75 2.5
# ODR-090 — a float mode is lexical. A parameter default keeps its declaring
# function's mode and a field default is strict, a struct's or a class's,
# wherever either is evaluated: the strict defaults a `@fastmath` function
# uses are computed in the main C file, and the `@fastmath` default a strict
# function uses in the `@fastmath` file (`[CG-C-11]`). A default doing no
# float work (`named`'s) is the same in every mode and is used in place.

struct Tile:
    w: f64
    h: f64 = 0.4375 * 2.0

class Gain:
    g: f64 = 0.15625 * 2.0

struct Scale:
    k: f64

    fn apply(self, x: f64, y: f64 = x * self.k + 0.0625) -> f64:
        return y

fn near(a: f64, eps: f64 = a * 0.001 + 1.0) -> f64:
    return eps

fn named(a: f64, b: f64 = a) -> f64:
    return b

@fastmath
fn wide(a: f64, b: f64 = a * 0.375 + 1.0) -> f64:
    return b

@fastmath
fn fast(a: f64):
    t = Tile(w = a)
    s = Scale(k = 0.5)
    gain = Gain()
    println(near(a), t.h, s.apply(a + 0.125), wide(a), named(1.0), gain.g)

fn main():
    fast(2.0)
    println(wide(2.0), named(2.5))

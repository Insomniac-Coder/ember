#$ test: run-pass
#$ rules: TYP-9, FN-5, CG-C-11
#$ profiles: debug, development, release
#$ assert-c-order: "8.125e-1" then "its `fastmath` functions"
#$ assert-c-order: "its `fastmath` functions" then "6.875e-1"
#$ stdout: 1.0 5.0 3.0
#$ stdout: 2.625
# ODR-090 — a `mut` parameter's default is a place, and it keeps its declaring function's float
# mode wherever it is evaluated, as any default does: a function of the declaration's mode hands
# the place back as a reference, taking what the default names by reference. `bump`'s strict
# default, used by a `@fastmath` caller, is computed in the main C file, and `bump_fast`'s
# `@fastmath` one, used by a strict caller, in the `@fastmath` file (`[CG-C-11]`). A `mut self`
# method's default reads `self` the same way.

class Holder:
    values: Array[f64] = []
    k: f64 = 0.5

fn bump(h: Holder, mut slot: f64 = h.values[(h.k * 0.8125 + 0.6) as int]):
    slot += 1.0

@fastmath
fn bump_fast(h: Holder, mut slot: f64 = h.values[(h.k * 0.6875 + 0.7) as int]):
    slot += 2.0

struct Meter:
    k: f64
    total: f64 = 0.0

    fn add(mut self, x: f64 = self.k * 0.40625 + 1.0):
        self.total += x

@fastmath
fn fast(h: Holder) -> f64:
    bump(h)
    m = Meter(k = 4.0)
    m.add()
    return m.total

fn main():
    h = Holder()
    h.values = [1.0, 2.0, 3.0]
    total = fast(h)
    bump_fast(h)
    println(h.values[0], h.values[1], h.values[2])
    println(total)

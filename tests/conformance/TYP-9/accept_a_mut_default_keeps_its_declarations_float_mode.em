#$ test: run-pass
#$ rules: TYP-9, FN-5, CG-C-11
#$ profiles: debug, development, release
#$ assert-c-order: "8.125e-1" then "its `fastmath` functions"
#$ assert-c-order: "its `fastmath` functions" then "6.875e-1"
#$ stdout: 1.0 2.0 3.0
#$ stdout: 5.625
#$ stdout: 4.0
# ODR-090, `[FN-5]` — a default keeps its declaring function's float mode wherever it is evaluated
# (a default is written for a plain or an `owns` parameter, and one on a `ref mut` parameter is
# `E3027`, so these take `owns`): `bump`'s strict default, used by a `@fastmath` caller, is computed
# in the main C file, and `bump_fast`'s `@fastmath` one, used by a strict caller, in the `@fastmath`
# file (`[CG-C-11]`). A method's default reads `this` the same way.

class Holder:
    values: Array[f64] = []
    k: f64 = 0.5

fn bump(h: Holder, owns slot: f64 = h.values[(h.k * 0.8125 + 0.6) as int]) -> f64:
    slot += 1.0
    return slot

@fastmath
fn bump_fast(h: Holder, owns slot: f64 = h.values[(h.k * 0.6875 + 0.7) as int]) -> f64:
    slot += 2.0
    return slot

struct Meter:
    k: f64
    total: f64 = 0.0

    fn add(rhs: f64 = this.k * 0.40625 + 1.0):
        this.total += rhs

@fastmath
fn fast(h: Holder) -> f64:
    a = bump(h)
    m = Meter(k = 4.0)
    m.add()
    return m.total + a

fn main():
    h = Holder()
    h.values = [1.0, 2.0, 3.0]
    total = fast(h)
    println(h.values[0], h.values[1], h.values[2])
    println(total)
    println(bump_fast(h))

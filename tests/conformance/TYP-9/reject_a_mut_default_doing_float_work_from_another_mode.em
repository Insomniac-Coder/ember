#$ test: compile-fail
#$ rules: TYP-9, FN-5, TYP-9c
#$ profiles: debug
#$ error[E0900]: the default of `slot` does float work in its function's `strict` mode, which a call from here cannot keep yet
# ODR-090 — a default keeps its declaration's float mode. A `mut` parameter's
# default is a place, which no closure of the declaration's mode can hand
# back, so a call from a `@fastmath` function whose default computes the
# place with float arithmetic is refused rather than computed in the wrong
# mode (`[TYP-9c]`: the attribute is never ignored). The strict call in `main`
# is fine.

struct Table:
    values: Array[f64]
    k: f64

    fn bump(mut self, mut slot: f64 = self.values[(self.k * 2.0) as int]):
        slot += 1.0

@fastmath
fn fast(mut t: Table):
    t.bump()

fn main():
    t = Table(values = [1.0, 2.0, 3.0], k = 0.5)
    t.bump()
    fast(t)
    println(t.values[1])

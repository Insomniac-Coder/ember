#$ test: run-pass
#$ rules: TYP-17, STD-17, STD-12
#$ profiles: debug, development, release
#$ stdout: 1 3
#$ {'a': 1, 'z': 7}
#$ 1.5 1.5 2.5
#$ [1.5, 3.5]
# `[TYP-17]` — `c[i]` on a type parameter is its bound's `Index.index`, a
# write its `IndexMut.index_mut`, and `c[k] = v` its `IndexSet.index_set`. A
# `Map` meets `Index[str]` (any key `Q: AsKey[K]`, `[STD-12]`), an `Array` and
# a `Span` `Index[int]` (ODR-042).

[C: needs Index[StrView, Output = int]]
fn lookup(c: C, k: StrView) -> int:
    return c[k]

[C: needs IndexSet[StrView, int]]
fn put(ref mut c: C, k: StrView):
    c[k] = 7

[C: needs Index[int, Output = f32]]
fn first(c: C) -> f32:
    return c[0]

[C: needs IndexMut[int, Output = f32]]
fn bump(ref mut c: C):
    c[1] += 1.0

fn main():
    m: Map[String, int] = {"a": 1, "c": 3}
    println(lookup(m, "a"), lookup(m, "c"))
    n: Map[String, int] = {"a": 1}
    put(n, "z")
    println(n)
    xs: Array[f32] = [1.5, 2.5]
    println(first(xs), first(xs.as_span()), xs[1])
    bump(xs)
    println(xs)

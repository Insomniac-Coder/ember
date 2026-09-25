#$ test: run-pass
#$ rules: TYP-17, STD-17, STD-12
#$ profiles: debug, release, shipping
#$ stdout: 1 3
#$ {'a': 1, 'z': 7}
#$ 1.5 1.5 2.5
#$ [1.5, 3.5]
# `[TYP-17]` — `c[i]` on a type parameter is its bound's `Index.index`, a
# write its `IndexMut.index_mut`, and `c[k] = v` its `IndexSet.index_set`. A
# `Map` meets `Index[str]` (any key `Q: AsKey[K]`, `[STD-12]`), an `Array` and
# a `Span` `Index[int]` (ODR-042).

fn lookup[C: Index[str, Output = int]](c: C, k: str) -> int:
    return c[k]

fn put[C: IndexSet[str, int]](mut c: C, k: str):
    c[k] = 7

fn first[C: Index[int, Output = f32]](c: C) -> f32:
    return c[0]

fn bump[C: IndexMut[int, Output = f32]](mut c: C):
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

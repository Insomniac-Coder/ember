#$ test: run-pass
#$ rules: STD-19
#$ profiles: debug, release, shipping
#$ stdout:
#$ [(0, 4), (1, 9), (2, 2), (3, 9)]
#$ 24 Some(9) 4 3 true
#$ 2 7 Some('b')
#$ Some(2) Some(2)
#$ 4,9,2,9
# `[STD-19]` — an adapter or consumer called on an `Iterable` borrows it as
# `iter()` would: `xs.enumerate()` is `xs.iter().enumerate()`, for a list, a
# map, a set and a view. A method of the collection's own of the same name
# comes first: `Array.join`.

fn main():
    xs: Array[int] = [4, 9, 2, 9]
    pairs: Array[(int, int)] = []
    for (i, x) in xs.enumerate():
        pairs.push((i, x))
    println(pairs)
    println(xs.sum(), xs.max(), xs.count(), xs.skip(1).count(), xs.any(fn(x) => x > 8))
    m: Map[str, int] = {"a": 1, "b": 2}
    s: Set[int] = {3, 4}
    println(m.count(), s.sum(), m.keys().last())
    view = xs.as_span()
    println(view.min(), view.position(fn(x) => x == 2))
    println(xs.join(","))

#$ test: run-pass
#$ rules: CLO-15, CLO-2, CLO-4, STD-19
#$ profiles: debug, development
#$ stdout: 3 [1, 2, 3] 9
#$ stdout: [4, 5, 6]
#$ stdout: 3 7
#$ stdout: ['a', 'b', 'c', 'd']
#$ stdout: ['1x', '2y']
# `[CLO-15]` (ODR-100) — the adapters take their callable over (`[STD-19]`),
# and a lambda written at the call keeps the captures `[CLO-2]` infers: what it
# writes lands in the caller's variable, which is the caller's again once the
# iterator is gone (`[CLO-4]`). An `owned fn` capturing a `String` goes through
# an adapter, fused in a `for` or called through `count`, and `zip` and `chain`
# take iterators that own their items.

fn main():
    xs = [1, 2, 3]
    seen: Array[int] = []
    total = 6
    k = xs.iter().inspect(fn(v: ref int) => seen.push(v)).count()
    m = xs.iter().filter(fn(v: ref int) => v < total).map(fn(v: ref int) => v * 1).sum()
    println(k, seen, m + 3)
    ys: Array[int] = []
    for y in xs.iter().map(fn(v: ref int) => v + seen.len()):
        ys.push(y)
    println(ys)
    s = "abcdefg".to_string()
    n = xs.iter().map(owned fn(v: ref int) => v + s.len()).count()
    t = "abcdefg".to_string()
    for z in xs.iter().map(owned fn(v: ref int) => v + t.len()).take(1):
        println(n, z - 1)
    left = ["a".to_string(), "b".to_string()]
    right = ["c".to_string(), "d".to_string()]
    joined: Array[String] = []
    for word in left.into_iter().chain(right.into_iter()):
        joined.push(word)
    println(joined)
    names = ["x".to_string(), "y".to_string()]
    pairs: Array[String] = []
    for p in xs.iter().zip(names.into_iter()):
        pairs.push(f"{p.0}{p.1}")
    println(pairs)

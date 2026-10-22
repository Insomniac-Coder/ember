#$ test: run-pass
#$ rules: STD-17, STD-15, SPN-1
#$ profiles: debug, development, release
#$ stdout: [1102, 104]
#$ [10]
## `[STD-17]` — `Map` implements `IndexMut`, so a mutable view of `m[k]` (`iter_mut`, a `for`
## over it with adapters, `chunks_mut`, `get_mut`) reaches the entry through `index_mut`, as
## `m[k].push(x)` does; it was `E3021`, "a shared reference, which only reads" (D-818).

struct Pair:
    a: Array[int]

fn main():
    m: Map[String, Array[int]] = {"x": [1, 2]}
    for v in m["x"].iter_mut():
        v += 1
    for i, v in m["x"].iter_mut().enumerate():
        v += i
    for c in m["x"].chunks_mut(1):
        c[0] += 100
    if Some(p) = m["x"].get_mut(0):
        p += 1000
    println(m["x"])
    ps: Map[int, Pair] = {1: Pair([5])}
    for v in ps[1].a.iter_mut():
        v *= 2
    println(ps[1].a)

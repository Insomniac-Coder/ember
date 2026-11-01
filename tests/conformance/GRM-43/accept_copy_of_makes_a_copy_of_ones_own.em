#$ test: run-pass
#$ rules: GRM-43, OWN-8, PHIL-3, GRM-44, GRM-13
#$ stdout: 5 6
#$ stdout: ab abc
#$ stdout: 3 2
#$ stdout: 1 9
#$ stdout: 7 8
#$ stdout: x xy
## `[GRM-43]`, `[OWN-8]` — `copy of e` makes a copy of the program's own with `Clone`, binding as a
## prefix `-` does: a number, a `String`, an `Array` and a struct marked `@derive(Clone)`. Changing
## the copy leaves the original as it was.
## `copy of v` after `exists as` and in a `match` branch names a copy of the part matched, which
## may be changed (`[GRM-44]`, `[GRM-13]`).

@derive(Clone)
struct Point:
    x: int = 0

fn main():
    a = 5
    b = copy of a
    b += 1
    println(a, b)
    s = String("ab")
    t = copy of s
    t.push_str("c")
    println(s, t)
    xs = [1, 2]
    ys = copy of xs
    ys.push(owns 3)
    println(ys.len(), xs.len())
    p = Point(x=1)
    q = copy of p
    q.x = 9
    println(p.x, q.x)
    held: Option[int] = Some(7)
    if held exists as copy of v:
        v += 1
        if held exists as w:
            println(w, v)
    named: Option[String] = Some(String("x"))
    match named:
        Some(copy of m):
            m.push_str("y")
            if named exists as n:
                println(n, m)
        None:
            pass

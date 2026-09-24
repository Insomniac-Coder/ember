#$ test: run-pass
#$ rules: EXP-5
#$ profiles: debug, release, shipping
#$ stdout: 9 [7, 2] 5 (1, 8)
# `[EXP-5]` — a local, an element, a field and a tuple field are places, and
# each can be assigned.

struct Point:
    x: int

fn main():
    n = 1
    n = 9
    xs = [1, 2]
    xs[0] = 7
    p = Point(x=1)
    p.x = 5
    t = (1, 2)
    t.1 = 8
    println(n, xs, p.x, t)

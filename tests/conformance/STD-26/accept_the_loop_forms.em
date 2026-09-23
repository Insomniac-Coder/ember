#$ test: run-pass
#$ rules: STD-26, CTL-3b
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ 1
#$ 2
#$ 2
#$ 3
#$ 4
#$ 10
#$ 7
#$ 4
#$ 1
#$ 0
#$ 4
#$ 8
#$ 0:3
#$ 1:1
#$ 2:4
#$ 1:3
#$ 2:1
#$ 3:4
#$ 3/10
#$ 1/20
#$ 4
#$ 1
#$ 3
# `range` with one, two and three arguments (a negative step counts down),
# `enumerate` (with `start`), `zip` (stops at the shorter) and `reversed`, as
# counted loops.

fn main():
    xs = [3, 1, 4]
    ys = [10, 20]
    for i in range(3):
        println(i)
    for i in range(2, 5):
        println(i)
    for i in range(10, 0, -3):
        println(i)
    for i in range(0, 10, 4):
        println(i)
    for i, x in enumerate(xs):
        println(f"{i}:{x}")
    for i, x in enumerate(xs, start=1):
        println(f"{i}:{x}")
    for a, b in zip(xs, ys):
        println(f"{a}/{b}")
    for x in reversed(xs):
        println(x)

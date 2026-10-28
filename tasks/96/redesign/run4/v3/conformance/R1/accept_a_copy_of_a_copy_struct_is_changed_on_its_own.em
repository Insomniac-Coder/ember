#$ test: run-pass
#$ rules: R1, R3
#$ stdout: 1 2
## R1, R3: the first way out: `q = copy of p` says the copy is your own; changing it leaves `p` as
## it was.

@derive(Copy)
struct Vec2:
    x: int
    y: int

fn main():
    p = Vec2(x=1, y=2)
    q = copy of p
    q.x += 1
    println(p.x, q.x)

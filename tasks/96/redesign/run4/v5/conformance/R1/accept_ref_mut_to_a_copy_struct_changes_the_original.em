#$ test: run-pass
#$ rules: R1, R2
#$ stdout: 2
## R1, R2: the second way out: `q = ref mut p` names `p` itself, so the change is `p`'s; `p` is read
## after `q`'s last use.

@derive(Copy)
struct Vec2:
    x: int
    y: int

fn main():
    p = Vec2(x=1, y=2)
    q = ref mut p
    q.x += 1
    println(p.x)

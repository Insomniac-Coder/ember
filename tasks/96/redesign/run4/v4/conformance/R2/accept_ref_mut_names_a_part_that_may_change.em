#$ test: run-pass
#$ rules: R2
#$ stdout: 2
## R2: a word before the part says otherwise: `pos = ref mut p.pos` is a name for that very place
## that may change it (AN-44's NEW).

@derive(Copy)
struct Vec2:
    x: int
    y: int

struct P:
    pos: Vec2

fn main():
    p = P(pos=Vec2(x=1, y=2))
    pos = ref mut p.pos
    pos.x += 1
    println(p.pos.x)

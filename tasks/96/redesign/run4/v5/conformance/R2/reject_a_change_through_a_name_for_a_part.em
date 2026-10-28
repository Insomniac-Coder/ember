#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to change `p`'s position write `pos = ref mut p.pos`; to change your own copy write `pos = copy of p.pos`
## design rules: R2, R5
## R2, R5: `pos = p.pos` names a part of `p` read-only (a struct with fields is viewed, not copied),
## so `pos.x += 1` is refused (E3023) with both words that would allow it (AN-44's message).

@derive(Copy)
struct Vec2:
    x: int
    y: int

struct P:
    pos: Vec2

fn main():
    p = P(pos=Vec2(x=1, y=2))
    pos = p.pos
    pos.x += 1                 #$ error[E3023]: `pos.x += 1` changes `p.pos` through `pos`, which only reads it
    println(p.pos.x)

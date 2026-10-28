#$ test: run-pass
#$ rules: R1
#$ stdout: 2
## R1: the E3023 rule for a `Copy` struct copied with `=` holds only while the name it was copied
## from is still used; here `p` is never used again, so `q` may change and nothing could be meant for
## `p`.

@derive(Copy)
struct Vec2:
    x: int
    y: int

fn main():
    p = Vec2(x=1, y=2)
    q = p
    q.x += 1
    println(q.x)

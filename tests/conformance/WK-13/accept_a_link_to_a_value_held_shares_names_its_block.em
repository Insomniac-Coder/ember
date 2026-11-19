#$ test: run-pass
#$ rules: WK-13, WK-11, WK-12, RC-1
#$ profiles: debug, development, release
#$ stdout: 2
#$ 3
#$ gone
# `[WK-13]` — a `weakref` link to a value held `shares` names the same block as its owners: opened
# while an owner lives it reads the value, with the changes the owners make; once the last owner
# ends, the link's weak count keeps the block and the opening finds it ended.

struct Score:
    points: int

fn main():
    owner = shares mut Score(points=2)
    w = weakref owner
    if w exists as s:
        println(s.points)
    owner.points = 3
    if w exists as s:
        println(s.points)
    owner = shares mut Score(points=9)
    if w exists as s:
        println(s.points)
    else:
        println("gone")

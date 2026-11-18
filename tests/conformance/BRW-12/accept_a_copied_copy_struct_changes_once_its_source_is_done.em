# Clause BRW-12.4: "a copied `Copy` struct is not then changed through the new name while the source
# is used" -- once the source is not used again, or with `copy of`, the copy may change.
#$ test: run-pass
#$ rules: BRW-12, OWN-7, OWN-8
#$ stdout: 9
#$ stdout: 1 8

@derive(Copy)
struct Point:
    x: int
    y: int

fn main():
    a = Point(x=1, y=1)
    b = a
    b.x = 9
    println(b.x)
    c = Point(x=1, y=1)
    d = copy of c
    d.x = 8
    println(c.x, d.x)

# Clause BRW-12.4: "and a copied `Copy` struct is not then changed through the new name while the
# source is used (`[OWN-7]`, `E3023`)."
#$ test: compile-fail
#$ rules: BRW-12, OWN-7
# The spec gives no message text for this case (`[OWN-7]` says only that the line says `copy of` or
# `ref mut`): codes only.

@derive(Copy)
struct Point:
    x: int
    y: int

fn main():
    pts = [Point(x=1, y=2)]
    p = pts[0]
    p.x = 5    #$ error[E3023]
    println(pts[0].x, p.x)
    a = Point(x=1, y=1)
    b = a
    b.y = 7    #$ error[E3023]
    println(a.y, b.y)

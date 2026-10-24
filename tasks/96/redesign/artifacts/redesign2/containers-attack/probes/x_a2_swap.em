# A2 attack 1: the swap idiom. Under the answer's rule 1, tmp names position i (the element is not a
# number), so `xs[j] = tmp` reads position i after it was replaced (a2/a2_models.c).
struct Vec2:
    x: float
    y: float

fn swap_pts(mut xs: Array[Vec2], i: int, j: int):
    tmp = xs[i]
    xs[i] = xs[j]
    xs[j] = tmp

fn main():
    ps = [Vec2(x=1.0, y=1.0), Vec2(x=2.0, y=2.0)]
    swap_pts(ps, 0, 1)
    println(ps[0].x)

#$ test: run-pass
#$ rules: ATT-1, STR-5, TYP-34, LT-1a
#$ profiles: debug, release, shipping
#$ stdout: 3 2
#$ 1
#$ 3
# Attributes from the table, on the declarations they apply to.

@derive(Copy, Clone)
struct Point:
    x: int
    y: int

@repr(u8)
enum Level:
    Low
    High

@view
struct Window:
    items: Span[int]

@borrows(xs)
fn pick_first(xs: Span[int], ys: Span[int]) -> Span[int]:
    return xs

fn main():
    p = Point(x = 3, y = 2)
    q = p
    println(q.x, p.y)
    level = Level.High
    println(level as u8)
    numbers = [1, 2, 3]
    w = Window(items = numbers)
    println(pick_first(w.items, w.items)[2])

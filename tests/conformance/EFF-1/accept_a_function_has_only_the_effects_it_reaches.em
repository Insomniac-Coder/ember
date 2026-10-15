#$ test: run-pass
#$ rules: EFF-1, EFF-5
#$ stdout: 6 3
# `[EFF-1]` — effects come only from what a function reaches: a drop that runs no code, and
# callees that neither allocate nor print, give none.

struct Point:
    x: int
    y: int

fn add(p: Point) -> int:
    return p.x + p.y

@noalloc
@noio
fn total(xs: Span[int]) -> int:
    sum = 0
    for x in xs:
        sum += add(Point(x, 0))
    return sum

fn main():
    values = [1, 2, 3]
    println(total(values.as_span()), add(Point(1, 2)))

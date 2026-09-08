#$ test: run-pass
#$ rules: BRW-4, BRW-1
#$ stdout: 10
#$ stdout: 20

## `[BRW-4]` — `ref mut a.x` and `ref mut a.y` may be live at once when `x` and
## `y` are distinct fields, and this holds through arbitrary nesting of field
## projections. Borrowing the *same* field twice, or reading the whole struct
## while a field of it is borrowed, still conflicts: the places overlap.

struct Point:
    x: i32
    y: i32

fn main():
    p: Point = Point(1, 2)
    a: ref mut i32 = ref mut p.x
    b: ref mut i32 = ref mut p.y
    a = 10
    b = 20
    println(p.x)
    println(p.y)

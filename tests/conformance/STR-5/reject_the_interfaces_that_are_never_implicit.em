#$ test: compile-fail
#$ rules: STR-5
## `[STR-5]` - `Copy`, `Ord`, `Hash`, `Default` and `Display` are never implicit on a struct: each is
## requested with `@derive(...)` or written by hand.

struct Pt:
    x: int
    y: int

fn main():
    a = Pt(1, 2)
    b = Pt(2, 1)
    println(Pt.default())    #$ error[E2020]: `Pt` has no associated function `default`
    println(a < b)    #$ error[E2040]: `Pt` does not implement `Ord`
    m: Map[Pt, int] = {}    #$ error[E2040]: `Pt` does not implement `std.collections.Hash`
    c = a
    println(a.x, c.x, m.len())    #$ error[E3040]: `a` was handed over to `c` on line 16 and is used on line 17

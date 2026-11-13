#$ test: compile-fail
#$ rules: STR-5
## `[STR-5]` - `Ord`, `Hash` and `Default` are never implicit on a struct (`Copy` is tested with `[STR-3]`): each is
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
    println(a.x, m.len())

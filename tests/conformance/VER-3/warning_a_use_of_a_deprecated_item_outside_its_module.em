#$ test: run-pass
#$ rules: VER-3
#$ profiles: debug
#$ stdout: 6 6 12 10 10 true
#$ 0
## `[VER-3]` (ODR-174) — `@deprecated(since, note)` on any item: each use of it outside its module
## is `W2192`, naming `since` and `note` (a call, a type, a construction, a method, a constant,
## an enum, through an import or through the module), and the program still compiles and runs.
## The uses inside `support.old` itself (`area_of` calls `area`) are not warned about.

import support.old
from support.old import area, area_of, Dims, Size, MAX, LIMIT, Shape

fn width(d: Dims) -> int:    #$ warning[W2192]: `support.old.Dims` is deprecated since 0.8: use `Size` instead
    return d.w

fn main():
    a = area(2, 3)    #$ warning[W2192]: `support.old.area` is deprecated since 0.9: use `area_of` instead
    b = old.area(3, 2)    #$ warning[W2192]: `support.old.area` is deprecated since 0.9
    s = Size(3, 4)
    d = Dims(5, 6)    #$ warning[W2192]: `support.old.Dims` is deprecated since 0.8
    println(a, b, s.surface(), MAX, LIMIT, width(d) == 5)    #$ warning[W2192]: `support.old.Size.surface` is deprecated since 0.9: use `area` instead
    #$ warning[W2192]: `support.old.MAX` is deprecated since 0.7: use `LIMIT` instead
    shape = Shape.Wide    #$ warning[W2192]: `support.old.Shape` is deprecated since 0.9: use `Size`
    match shape:
        Shape.Square:    #$ warning[W2192]: `support.old.Shape` is deprecated since 0.9
            pass
        Shape.Wide:    #$ warning[W2192]: `support.old.Shape` is deprecated since 0.9
            pass
    println(area_of(1, 1) + s.area() - 13)

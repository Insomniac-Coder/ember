#$ test: run-pass
#$ rules: VER-3, ATT-1
#$ stdout: 6 2 3
## `[VER-3]` (ODR-174) — `@deprecated(since, note)` applies to any item, by name or by position,
## and a use inside the item's own module is not warned about: the module may keep using what it
## deprecates while its callers move. The deprecated items of another module, used only through
## items that are not deprecated, give no warning either.

from support.old import area_of, LIMIT

@deprecated(since="1.1", note="use `Point` instead")
struct Pair:
    a: int
    b: int

@deprecated("1.0", "use `scale` instead")
fn twice(x: int) -> int:
    return x * 2

struct Point:
    x: int

    @deprecated(since="1.2", note="read `x`")
    fn get(self) -> int:
        return self.x

fn main():
    p = Pair(1, 2)
    println(area_of(2, 3), Point(twice(p.a)).get(), LIMIT // 3)

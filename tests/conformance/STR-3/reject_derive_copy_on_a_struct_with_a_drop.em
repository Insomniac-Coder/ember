#$ test: compile-fail
#$ rules: STR-3, DRV-1
# `[STR-3]` — `@derive(Copy)` on a type that declares `drop` is `E2080`: the `drop` would run once
# per copy. The message names both lines.

@derive(Copy) #$ error[E2080]: `Handle` has a `drop` (line 10), which would run once per copy; remove `@derive(Copy)` (line 6), and hand a `Handle` over or write `copy of h` with a `clone` of its own
struct Handle:
    id: int

    fn drop():
        pass

fn main():
    h = Handle(id=1)
    println(h.id)

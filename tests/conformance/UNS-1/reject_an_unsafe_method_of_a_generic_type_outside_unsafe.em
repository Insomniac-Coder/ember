#$ test: compile-fail
#$ rules: UNS-1
#$ error[E3100]: `c.read_raw()` (line 16) calls an `unsafe fn`, whose callers must make true what its `@safety` says
# D-710 — an `unsafe fn` method of a generic type is `unsafe` in every instance: calling it
# outside `unsafe` is refused, as for any other type.

struct Cell2[T]:
    value: T

    @safety("the caller has checked the value")
    unsafe fn read_raw(self) -> int:
        return 1

fn main():
    c = Cell2[int](value = 2)
    println(c.read_raw())

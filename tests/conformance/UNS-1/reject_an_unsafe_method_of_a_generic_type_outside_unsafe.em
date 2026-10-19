#$ test: compile-fail
#$ rules: UNS-1
#$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block
# D-710 — an `unsafe fn` method of a generic type is `unsafe` in every instance: calling it
# outside `unsafe` is refused, as for any other type.

struct Cell2[T]:
    value: T

    @safety("the caller has checked the value")
    unsafe fn read_raw(self) -> int:  # SAFETY: the caller's obligation
        return 1

fn main():
    c = Cell2[int](value = 2)
    println(c.read_raw())

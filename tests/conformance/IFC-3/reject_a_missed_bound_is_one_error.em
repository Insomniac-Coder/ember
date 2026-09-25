#$ test: compile-fail
#$ rules: IFC-3, TYP-17, DIA-14
# `[IFC-3]`, `[DIA-14]` — a type that misses a bound misses the parents it
# brings too, and that is one mistake: `Plain` does not implement `Whole`,
# said once, not again for `Default` and `Hash` (D-337).

interface Whole: Default + Hash:
    pass

fn zero_of[T: Whole](v: T) -> T:
    return T.default()

struct Plain:
    x: i32

fn main():
    _p = zero_of(Plain(1)) #$ error[E2040]: `Plain` does not implement `Whole`, which `T` requires

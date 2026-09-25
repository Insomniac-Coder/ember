#$ test: compile-fail
#$ rules: TYP-38, GRM-26
# ODR-034 — `{}` is an empty `Map`; an empty set is written `Set[T]()`.

fn main():
    s: Set[int] = {}  #$ error[E2020]: `{}` is an empty `Map`, not a `Set`

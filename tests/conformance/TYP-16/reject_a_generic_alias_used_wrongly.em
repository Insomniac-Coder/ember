#$ test: compile-fail
#$ rules: TYP-16
#$ error[E2020]: `Pair` takes 1 type argument, found 2
#$ error[E2040]: `f64` does not implement `Hash`, which `Index` requires of `K`
# `[TYP-16]` — a generic alias takes as many arguments as it has parameters, and each meets its
# parameter's bounds.

type Pair[T] = (T, T)
type Index[K: Hash] = Map[K, int]

fn main():
    a: Pair[int, int] = (1, 2)
    b: Index[float] = {}

#$ test: compile-fail
#$ rules: TYP-36, HASH-4
# D-318 — a float range is no `Hash` (`[TYP-36]`), so it is no map key.

type R = f32 in 0.25 ..= 0.75

fn main():
    m: Map[R, int] = {}    #$ error[E2040]: `R` does not implement `std.collections.Hash`, which `Map`'s `K` requires
    println(m.len())

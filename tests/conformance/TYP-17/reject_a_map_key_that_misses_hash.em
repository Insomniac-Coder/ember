#$ test: compile-fail
#$ rules: TYP-17, STD-11
# D-301 — a generic type's arguments meet the bounds its parameters declare,
# checked where the type is named: `Map[K, V]` needs `K: Eq + Hash`, and a
# float is not `Hash` ([HASH-3]). The errors used to appear only inside
# std's `Map` methods, with nothing on the line that named the type.

struct Table:
    m: Map[f64, int]    #$ error[E2040]: `f64` does not implement `std.collections.Hash`, which `Map`'s `K` requires

fn main():
    s = {1.5, 2.5}    #$ error[E2040]: `f64` does not implement `std.collections.Hash`, which `Set`'s `T` requires
    m: Map[f64, int] = {}    #$ error[E2040]: `f64` does not implement `std.collections.Hash`, which `Map`'s `K` requires
    ok: Map[int, f64] = {1: 1.5}
    println(len(s), ok)

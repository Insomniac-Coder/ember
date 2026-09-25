#$ test: compile-fail
#$ rules: HASH-1, TYP-36, DRV-1
# `[DRV-1]` — a derive whose requirement a field does not meet is `E2080`,
# naming the field: floats are not `Hash` (`[TYP-36]`). A struct that does not
# derive `Hash`, and a tuple holding a float, fail a `Hash` bound (`E2040`).

from std.collections import Hash, Hasher, DefaultHasher

@derive(Hash)
struct Sample:
    at: int
    value: f64  #$ error[E2080]: field `value` has type `f64`, which is not Hash

struct Plain:
    x: int

fn hash_of[K: Hash](k: K) -> u64:
    h = DefaultHasher.new()
    k.hash(h)
    return h.finish()

fn main():
    println(hash_of(Plain(1)))  #$ error[E2040]
    println(hash_of((1, 2.0)))  #$ error[E2040]

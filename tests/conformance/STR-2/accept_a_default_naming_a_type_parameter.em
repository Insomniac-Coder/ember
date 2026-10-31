#$ test: run-pass
#$ rules: STR-2, TYP-16, TYP-17
#$ stdout: 0 1
# `[STR-2]` — a generic struct's field default reads as its declaration does,
# so it may name the struct's own type parameters: they mean the instance's
# arguments, and a bound's members are available (`H.default()`).

from std.collections import Hasher, DefaultHasher

[K]
struct Inner:
    keys: Array[K] = []

[K, H: needs (Hasher, Default)]
struct B:
    keys: Array[K] = []
    seed: H = H.default()
    inner: Inner[K] = Inner[K]()

fn main():
    b = B[i32, DefaultHasher]()
    b.inner.keys.push(4)
    println(b.keys.len(), b.inner.keys.len())

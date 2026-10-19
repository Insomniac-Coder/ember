#$ test: run-pass
#$ rules: HND-2
#$ stdout: 8 4
#$ stdout: 1 0 1 0
#$ stdout: 0 1
# `[HND-2]` — a `Handle[T]` is a `u64` of a 32-bit index and a 32-bit generation; a
# `Pool[T, Split20]` gives a `u32` of a 20-bit index and a 12-bit generation.

from std.collections import Pool, Handle, Split20

fn main():
    println(size_of[Handle[int]](), size_of[Handle[int, Split20]]())
    wide = Pool[int]()
    small = Pool[int, Split20]()
    _a = wide.insert(1)
    b = wide.insert(2)
    _c = small.insert(1)
    d = small.insert(2)
    println(b.index(), b.generation(), d.index(), d.generation())
    small.remove(d)
    e = small.insert(3)
    println(e.index() - d.index(), e.generation())

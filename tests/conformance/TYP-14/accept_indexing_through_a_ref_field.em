#$ test: run-pass
#$ rules: TYP-14, SPN-2
#$ stdout: 1 [1]
#$ stdout: [5, 7]
# `[TYP-14]` — a reference is read through where a value is wanted, so a
# `ref Array` field indexes and slices as a `ref Array` local does, and a
# `ref mut Array` field is written through.

struct View[K]:
    owner: ref Array[K]

struct Edit:
    target: ref mut Array[int]

fn main():
    xs = [1, 2]
    v = View(ref xs)
    println(v.owner[0], v.owner[0..1])
    ys = [5, 6]
    e = Edit(ref mut ys)
    e.target[1] = 7
    println(ys)

#$ test: compile-fail
#$ rules: BRW-5, BRW-1
# The hazard the disjointness exemption must not open. `push` may reallocate,
# which would leave the element borrow pointing at freed memory — and it takes
# `mut v` with an empty projection, so it overlaps every place under `v`
# including a disjoint constant index.

fn main():
    v: Array[i32] = Array[i32]()
    v.push(1)
    a: ref mut i32 = ref mut v[0]
    v.push(2)              #$ error[E3022]: `v[0]` is already mutably borrowed
    a = 10
    println(v[0])

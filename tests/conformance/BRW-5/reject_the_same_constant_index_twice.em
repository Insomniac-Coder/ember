#$ test: compile-fail
#$ rules: BRW-5, DIA-7a
# "constants and **different**" — the same constant twice is the same place,
# and `[DIA-7a]` keys this to shape B1, "two mutable indices".

fn main():
    v: Array[i32] = Array[i32]()
    v.push(owns 1)
    a: ref mut i32 = ref mut v[0]
    b: ref mut i32 = ref mut v[0]     #$ error[E3022]: `b` and `a` would both change an element of `v`, but both are `v[0]`
    a = 10
    b = 20
    println(v[0])

#$ test: compile-fail
#$ rules: BRW-4, BRW-10, DIA-7a
# `[BRW-10]`: what a method does to its object is worked out from its body (`E3025`, shape B8,
# is retired). `set_x` changes `this.x`, so `p.set_x(10)` changes `p.x` while `a`, a `ref mut` name
# of that field, is still used: one place changed through two names (`E3022`, shape B1). A call of
# a method that changes only the other field is accepted (`accept_two_disjoint_fields.em`,
# `BRW-10`).

struct P:
    pub x: i32
    pub y: i32
    fn set_x(v: i32):
        this.x = v

fn main():
    p = P(1, 2)
    a: ref mut i32 = ref mut p.x
    p.set_x(10)   #$ error[E3022]: `p.x` is already mutably borrowed
    println(a)

#$ test: compile-fail
#$ rules: BRW-4, DIA-7a
# Two MUTABLE borrows of one field is B1, not B3: `[DIA-7a]` keys it to
# `E3022`. (Reported `E3021` before the reservation-window fix — a borrow in a
# user local was treated as reserved merely because the local was later used
# as a call argument. D-040 keeps the remaining half: a METHOD call defeating
# disjointness still reports `E3022`, where the keying wants `E3025`.)

struct P:
    pub x: i32
    pub y: i32

fn main():
    p = P(1, 2)
    a: ref mut i32 = ref mut p.x
    b: ref mut i32 = ref mut p.x   #$ error[E3022]: `p.x` is already mutably borrowed
    println(a)
    println(b)

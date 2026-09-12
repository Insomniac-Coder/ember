#$ test: compile-fail
#$ rules: CELL-10, BRW-1, BRW-4, DIA-7a, DIA-9
#$ not-help: RefCell
# `[CELL-10]` forbids a RefCell suggestion for simultaneous conflicting
# accesses in one expression. Two live mutable borrows are shape B1, so the
# ordinary exclusivity diagnostic must not advertise interior mutability.

fn main():
    value: i32 = 1
    first: ref mut i32 = ref mut value
    second: ref mut i32 = ref mut value  #$ error[E3022]: `value` is already mutably borrowed
    println(first)
    println(second)

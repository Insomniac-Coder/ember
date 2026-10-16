#$ test: compile-fail
#$ rules: EFF-6a, EFF-12
#$ error[E4030]: a `@static_safe` function takes `cell`, a `RefCell[i64]`
#$ error[E4030]: a `@static_safe` function takes `guard`, a `RefMut[i64]`
# `[EFF-6a]` — a `@static_safe` function may not take a parameter whose type forces a dynamic
# check at the call site: a `RefCell`, or a guard a `RefCell` borrow check made. A plain value or a
# `ref mut` to one is proved when it compiles.

@static_safe
fn read(cell: RefCell[int]) -> int:
    return 0

@static_safe
fn bump(mut guard: RefMut[int]):
    pass

@static_safe
fn twice(mut x: int):
    x = x * 2

fn main():
    y = 3
    twice(y)
    println(y)

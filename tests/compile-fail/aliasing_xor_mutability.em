#$ test: compile-fail
#$ rules: BRW-1, BRW-2, DIA-3
#$ error[E3022]: `n` is already mutably borrowed
#$ error[E3021]: `m` cannot be read while it is mutably borrowed

## `[BRW-1]` — at any point a place has either any number of shared borrows or
## exactly one mutable borrow, and while a mutable borrow is live the owner may
## not read, write, move or drop it.

fn two_mutable_borrows() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    s: ref mut i32 = ref mut n
    r = 2
    s = 3
    return n

fn read_while_mutably_borrowed() -> i32:
    m: i32 = 1
    r: ref mut i32 = ref mut m
    ## `m` is read here while `r` is still live, because `r` is used below.
    copy: i32 = m
    r = 2
    return copy

fn main():
    println(two_mutable_borrows())
    println(read_while_mutably_borrowed())

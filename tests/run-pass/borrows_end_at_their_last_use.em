#$ test: run-pass
#$ rules: BRW-2
#$ stdout: 6
#$ stdout: 3

## `[BRW-2]` — a borrow is live from its creation to the **last use** of what
## derives from it, not to the end of its scope. Every program here is rejected
## by a scope-based rule and accepted by this one; that difference is the whole
## reason the analysis is a liveness computation.

fn two_borrows_in_sequence() -> i32:
    n: i32 = 1
    r: ref mut i32 = ref mut n
    r = 5
    ## `r` is dead from here, so a second mutable borrow is fine.
    s: ref mut i32 = ref mut n
    s = 6
    return n

fn read_after_the_borrow_dies() -> i32:
    n: i32 = 3
    r: ref i32 = ref n
    total: i32 = r
    ## `r` is dead, so the owner may be read and written again.
    n = total
    return n

fn main():
    println(two_borrows_in_sequence())
    println(read_after_the_borrow_dies())

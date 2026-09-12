#$ test: run-pass
#$ rules: ARN-1, ARN-4, LT-4
# A growing Arena takes a shared borrow for allocation, so two returned mutable
# views may remain live together. Growth must not move the first allocation.
# Starting with four bytes lets the first allocation fill the initial chunk
# and forces the second allocation into another one.

fn main():
    arena = Arena.with_capacity(4)
    first: ref mut i32 = arena.alloc(10)
    second: ref mut i64 = arena.alloc(20i64)
    first = 11
    second = 22
    println(first)
    println(second)
#$ stdout: 11
#$ 22
#$ assert-c: contains("ember_arena_alloc")
#$ assert-c: contains("ember_arena_free")

#$ test: run-pass
#$ rules: LT-4, ARN-1
# The allocation view ends before its arena is dropped.

fn main():
    arena = Arena.with_capacity(16)
    stored: ref mut i32 = arena.alloc(8)
    println(stored)
#$ stdout: 8

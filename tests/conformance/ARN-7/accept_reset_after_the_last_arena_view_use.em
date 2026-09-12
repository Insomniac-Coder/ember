#$ test: run-pass
#$ rules: ARN-7, BRW-2
# A reset is permitted once NLL has ended every view into the arena.

fn main():
    arena = Arena.with_capacity(16)
    first: ref mut i64 = arena.alloc(41i64)
    println(first)
    arena.reset()
    second: ref mut i64 = arena.alloc(42i64)
    println(second)
#$ stdout: 41
#$ 42

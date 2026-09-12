#$ test: run-pass
#$ rules: ARN-4, ARN-7, BRW-2
# NLL ends the first view at its last use. reset can then rewind the chunks and
# a later allocation reuses the arena safely.

fn main():
    arena = Arena.with_capacity(16)
    first: ref mut i64 = arena.alloc(41i64)
    println(first)
    arena.reset()
    second: ref mut i64 = arena.alloc(42i64)
    println(second)
#$ stdout: 41
#$ 42

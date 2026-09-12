#$ test: run-pass
#$ rules: ARN-2, ARN-3
# Plain values need no individual destructor and use ordinary alloc.

fn main():
    arena = Arena.with_capacity(16)
    stored: ref mut i32 = arena.alloc(6)
    println(stored)
#$ stdout: 6

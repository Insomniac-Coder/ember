#$ test: run-fail
#$ rules: ARN-4
# A fixed arena may never grow. Exhaustion is a panic rather than an allocation
# or an out-of-bounds write, in every profile.

fn main():
    bytes: [u8; 4] = [0, 0, 0, 0]
    fixed: FixedArena = Arena.fixed(bytes)
    first: ref mut i32 = fixed.alloc(1)
    second: ref mut i32 = fixed.alloc(2)
    println(first + second)
#$ panics: fixed arena exhausted

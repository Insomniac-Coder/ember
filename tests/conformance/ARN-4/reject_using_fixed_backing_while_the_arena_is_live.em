#$ test: compile-fail
#$ rules: ARN-4, LT-4, BRW-1
# FixedArena is a view over the supplied MutSpan. The backing bytes stay
# exclusively borrowed for as long as the fixed arena is used.

fn main():
    bytes: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0]
    fixed: FixedArena = Arena.fixed(bytes)
    bytes[0] = 1              #$ error[E3021]: `bytes` cannot be written while it is borrowed
    stored: ref mut i32 = fixed.alloc(2)
    println(stored)

#$ test: run-pass
#$ rules: ARN-4
#$ stdout: 42
# `[ARN-4]` — a `FixedArena` never grows and carries no `Alloc` on allocation: it is the arena for
# `@noalloc` code.

@noalloc
fn place(mut fixed: FixedArena, x: i32) -> i32:
    slot: ref mut i32 = fixed.alloc(x)
    slot += 1
    return slot

fn main():
    bytes: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    fixed: FixedArena = Arena.fixed(bytes)
    println(place(fixed, 41))

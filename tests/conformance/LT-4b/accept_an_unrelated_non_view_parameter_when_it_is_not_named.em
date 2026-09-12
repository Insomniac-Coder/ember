#$ test: run-pass
#$ rules: LT-4a, LT-4b
# The exception is tied to the named Arena only. Other ordinary value
# parameters remain ordinary values and need no artificial region.

@borrows(arena)
fn allocate(arena: Arena, count: usize) -> ref mut usize:
    return arena.alloc(count)

fn main():
    arena = Arena.with_capacity(16)
    value = allocate(arena, 3)
    println(value)
#$ stdout: 3

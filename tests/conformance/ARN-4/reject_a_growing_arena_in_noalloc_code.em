#$ test: compile-fail
#$ rules: ARN-4
#$ error[E4001]: an arena allocation, which may grow the arena
# `[ARN-4]` — a growing `Arena` carries `Alloc` on its growth path.

@noalloc
fn place(mut arena: Arena, x: int) -> int:
    r = arena.alloc(x)
    return r

fn main():
    arena = Arena.with_capacity(64)
    println(place(arena, 3))

#$ test: run-pass
#$ rules: LT-44, LT-4
#$ stdout: 1
# `[LT-44]` — a function whose only view-producing parameter is one arena
# returns allocations that borrow it, with no `@borrows`.

fn allocate_one(arena: Arena) -> ref mut i32:
    return arena.alloc(1)

fn main():
    arena = Arena.with_capacity(16)
    stored = allocate_one(arena)
    println(stored)

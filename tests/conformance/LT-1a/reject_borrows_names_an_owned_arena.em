#$ test: compile-fail
#$ rules: LT-1a, LT-44
# ODR-024 — an `owned` parameter that is not a view is not a source: the
# function drops it, so nothing it returns can borrow it.

@borrows(arena)    #$ error[E2031]: `@borrows` names `arena`, which a result cannot borrow
fn fresh(owned arena: Arena) -> ref mut int:
    return arena.alloc(1)

fn main():
    println(0)

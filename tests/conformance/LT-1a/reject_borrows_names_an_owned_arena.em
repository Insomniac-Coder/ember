#$ test: compile-fail
#$ rules: LT-1a, LT-44
# ODR-024 — an `owned` parameter that is not a view is not a source: the
# function drops it, so nothing it returns can borrow it.

#$ error[E2031]: `from` names `arena`, which a result cannot point into
fn fresh(owns arena: Arena) -> ref mut int from arena:
    return arena.alloc(1)

fn main():
    println(0)

#$ test: compile-fail
#$ rules: TYP-38
# ODR-034 — `[…]` is always a list: in a `Set` position it is `E2020`, whose
# help writes the braces.

fn main():
    s: Set[int] = [1, 2]  #$ error[E2020]: a list literal is not a `Set`

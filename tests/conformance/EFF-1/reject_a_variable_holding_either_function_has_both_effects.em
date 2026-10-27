#$ test: compile-fail
#$ rules: EFF-1, FN-6, TYP-23
# ODR-278, `[EFF-1]` — a variable that may hold either of two functions holds their shape,
# `fn(int) -> int`, and a call through it runs whichever it holds: it has both functions'
# effects, so a `@noalloc` function calling through it reaches `widths`' allocation. Each of
# the three ways two functions meet makes the shape: a conditional expression, an assignment
# in each branch, and a second assignment.

fn double(x: int) -> int:
    return x * 2

fn widths(x: int) -> int:
    return [x, x].len()

@noalloc
fn either(c: bool, x: int) -> int:
    f = double if c else widths
    return f(x)  #$ error[E4001]: `@noalloc` function `either` reaches an allocation

@noalloc
fn branches(c: bool, x: int) -> int:
    if c:
        g = double
    else:
        g = widths
    return g(x)  #$ error[E4001]: `@noalloc` function `branches` reaches an allocation

@noalloc
fn reassigned(x: int) -> int:
    h = double
    h = widths
    return h(x)  #$ error[E4001]: `@noalloc` function `reassigned` reaches an allocation

fn main():
    println(either(true, 1), branches(false, 2), reassigned(3))

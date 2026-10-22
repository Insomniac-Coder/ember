#$ test: compile-fail
#$ rules: FN-2a, CLO-2
#$ help: bind the value to a local first
## `[FN-2a]` — a lambda written as the argument of a `mut` callable parameter is a value, not the
## mutable place the parameter borrows: shape B10, as for any other value. It compiled, and the
## callee read the closure as a pointer to one (the C did not compile).

fn feed(mut f: fn(int), n: int):
    for i in range(n):
        f(i)

fn main():
    total = 0
    feed(fn(i) => println(i + total), 2)  #$ error[E3027]: a `mut` argument is not a mutable place

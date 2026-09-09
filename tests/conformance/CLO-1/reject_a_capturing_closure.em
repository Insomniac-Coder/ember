#$ test: compile-fail
#$ rules: CLO-1, CLO-2
## A closure that captures is "a unique anonymous struct implementing
## `Callable`" and "a **view type** if it captures anything by reference".
## Neither is built yet, and the diagnostic says so rather than reporting that
## the captured name cannot be found — which would send the reader looking for
## a typo.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    scale = 3
    println(apply(fn(x) => x * scale, 2))    #$ error[E1010]: this closure captures `scale`

#$ test: compile-fail
#$ rules: FN-6, TYP-16
## A generic is a recipe, not a function: `[TYP-18]` needs the arguments to
## pick one, so there is no single signature to be a value of.

fn identity[T](x: T) -> T:
    return x

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    println(apply(identity, 1))     #$ error[E2060]: `identity` is generic, so it is not one function

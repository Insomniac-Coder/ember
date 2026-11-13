#$ test: compile-fail
#$ rules: GRM-8b
## `[GRM-8b]` — an instantiation argument that is neither a type nor a constant is `E2173`: a local
## variable is a value that exists only while the program runs, and an associated-type binding is not
## an argument of a function's instantiation.

[T]
fn id(x: T) -> T:
    return x

fn main():
    n = 3
    a = id[n](1)    #$ error[E2173]: `n` is a variable, which is neither a type nor a constant
    b = id[Item = int](1)    #$ error[E2173]: `Item = …` binds an associated type
    println(a, b)

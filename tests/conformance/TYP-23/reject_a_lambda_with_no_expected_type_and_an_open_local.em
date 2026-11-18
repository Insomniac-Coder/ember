#$ test: compile-fail
#$ rules: TYP-23
## `[TYP-23]` - a lambda with unannotated parameters and no expected callable type is `E2061`.

fn main():
    g = fn(a) => a    #$ error[E2061]: cannot tell what `a` holds
    println(1)

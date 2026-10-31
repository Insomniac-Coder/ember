#$ test: compile-fail
#$ rules: FN-8
# ODR-101 — nothing calls `main` that could choose its type arguments.

[T]
fn main(): #$ error[E2183]: it has type parameters
    println("generic")

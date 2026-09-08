#$ test: compile-fail
#$ rules: GRM-8a, GRM-8b
#$ error[E2172]: cannot index with a type

## `[GRM-8a]` — the parser commits to a type inside `[…]` on the seven tokens
## that cannot begin an expression, so `xs[ref i32]` parses. `[GRM-8b]` — name
## resolution then finds an index, and says so about the argument rather than
## reporting a type mismatch about the whole expression.

fn main():
    xs: Array[i32] = Array[i32]()
    n: i32 = xs[ref i32]
    println(n)

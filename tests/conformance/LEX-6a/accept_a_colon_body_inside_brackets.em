#$ test: run-pass
#$ rules: LEX-6a, GRM-16
## "Inside brackets, a lambda's `:` body is a single `small_stmt`, terminated
## by the enclosing closing bracket or by a `,` at the same bracket depth. No
## `NEWLINE` is required or emitted."
##
## `[GRM-16]` makes `return` an expression, so `fn(x): return e` and
## `fn(x) => e` are the same body.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    println(apply(fn(x: i32): return x * 2, 4))
    println(apply(fn(x: i32) => x * 2, 4))
#$ stdout: 8
#$ 8

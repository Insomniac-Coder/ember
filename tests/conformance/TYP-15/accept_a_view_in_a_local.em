#$ test: run-pass
#$ rules: TYP-15
## "Views MAY live in locals, parameters, return values, and in tuple, enum,
## `Option` and `Result` payloads."

fn head(xs: Span[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(5)
    v: Span[i32] = a
    println(head(v))
#$ stdout: 5

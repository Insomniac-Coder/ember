#$ test: compile-fail
#$ rules: ERR-2
# `[ERR-2]` — `?` in a function returning neither an `Option` nor a `Result`
# is `E2180`.

fn maybe(n: int) -> Option[int]:
    if n > 0:
        return Some(n)
    return None

fn first(n: int) -> int:
    v = maybe(n)?    #$ error[E2180]: needs the function to return one too, not `i64`
    return v

fn main():
    println(first(1))

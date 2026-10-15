#$ test: run-pass
#$ rules: EFF-5, EFF-14, PAN-2
#$ stdout: 6
# `[EFF-5]` — a function that has none of the effects its contracts forbid is accepted; they
# compose (`[EFF-14]`), and `panic("literal")` allocates nothing (`[PAN-2]`).

@static_safe
@noalloc
@nosync
@noio
@noblock
@nolock
@deterministic
fn total(xs: Span[int]) -> int:
    sum = 0
    for x in xs:
        sum += x
    return sum

@noalloc
fn refuse(n: int) -> int:
    if n < 0:
        panic("negative")
    return n

fn main():
    values = [1, 2, 3]
    println(total(values.as_span()) + refuse(0))

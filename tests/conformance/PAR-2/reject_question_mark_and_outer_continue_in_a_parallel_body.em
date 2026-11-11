#$ test: compile-fail
#$ rules: PAR-2
#$ error[E7010]: `?` on line 14 would return from `parse_all` in the middle of the parallel loop; keep each result instead: write `out[i] = parse(i)` into an `Array` of the results and check them after the loop
#$ error[E7010]: a `@parallel` loop's body may not `continue` an outer loop
# `[PAR-2]` — the body of a `@parallel` loop has no `?` (it would return from the function in the
# middle of the loop) and no `continue` naming an outer loop.

fn parse(i: int) -> Result[int, int]:
    return Ok(i)

fn parse_all(ref mut out: MutSpan[int]) -> Result[int, int]:
    @parallel
    for i in 0..out.len():
        out[i] = parse(i)?
    return Ok(0)

fn skips(ref mut out: MutSpan[int]):
    outer: for r in 0..3:
        @parallel
        for i in 0..out.len():
            if out[i] < 0:
                continue outer
            out[i] = 1

fn main():
    pass

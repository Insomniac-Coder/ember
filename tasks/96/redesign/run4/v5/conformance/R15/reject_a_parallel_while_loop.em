#$ test: compile-fail
#$ rules: PAR-1
## design rules: R15
## R15: "A `@parallel` loop splits a range or a list walked by element into chunks (E7016 for anything
## else)" (section 3's message; AN-84).

fn place(ref mut xs: MutSpan[int], idx: Span[int]):
    i = 0
    @parallel
    while i < idx.len():    #$ error[E7016]: `@parallel` splits a range or a list into chunks, and `while i < idx.len():` (line 9) in `place` is neither; instead of `i = 0`, the `while` line and `i += 1`, write `for i in 0..idx.len():`
        xs[i] = idx[i]
        i += 1

fn main():
    a = [0, 0, 0]
    order = [2, 0, 1]
    place(ref mut a.as_mut_span(), order.as_span())
    println(a)

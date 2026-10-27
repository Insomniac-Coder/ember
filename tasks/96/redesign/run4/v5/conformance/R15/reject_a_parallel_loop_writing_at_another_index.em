#$ test: compile-fail
#$ rules: PAR-2
#$ help: run the loop on one thread: remove `@parallel` (line 7)
## design rules: R15
## R15: "E7010 where chunks may write one place" (AN-84; the message gives the line that compiles, R18).

fn scatter(ref mut xs: MutSpan[int], idx: Span[int]):
    @parallel
    for i in 0..idx.len():
        xs[idx[i]] = i    #$ error[E7010]: the parallel loop writes `xs[idx[i]]`, and two rounds may have the same `idx[i]`, so two threads could write one element; run the loop on one thread: remove `@parallel` (line 7)

fn main():
    a = [0, 0, 0]
    order = [2, 0, 1]
    scatter(ref mut a.as_mut_span(), order.as_span())
    println(a)

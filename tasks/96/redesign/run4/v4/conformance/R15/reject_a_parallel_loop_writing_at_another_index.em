#$ test: compile-fail
#$ rules: R15
#$ help: index it by the loop's own index, `i + k`, or make it a reduction
## R15: "E7010 where chunks may write one place" (today's message, kept: AN-84).

fn scatter(ref mut xs: MutSpan[int], idx: Span[int]):
    @parallel
    for i in 0..idx.len():
        xs[idx[i]] = i    #$ error[E7010]: the parallel loop writes `xs`, and reaches it here at an index that is not `i` plus a constant

fn main():
    a = [0, 0, 0]
    order = [2, 0, 1]
    scatter(ref mut a.as_mut_span(), order.as_span())
    println(a)

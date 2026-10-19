#$ test: run-pass
#$ rules: PAR-1, PAR-2
#$ profiles: debug, development
#$ stdout: 99990000
#$ stdout: 30
# `[PAR-1]` — a `@parallel` loop's body runs over chunks of its range on the job system's
# workers and the calling thread; each iteration writes its own element of a `MutSpan`, at `i`.

fn main():
    n = 10000
    source: Array[int] = []
    out: Array[int] = []
    for i in range(n):
        source.push(i)
        out.push(0)
    xs = out.as_mut_span()
    @parallel(chunk=100)
    for i in 0..n:
        xs[i] = source[i] * 2
    total = 0
    for v in out:
        total += v
    println(total)
    small = [1, 2, 3, 4]
    ys = small.as_mut_span()
    @parallel
    for i in 0..=3:
        ys[i] = ys[i] * 3
    println(small[0] + small[1] + small[2] + small[3])

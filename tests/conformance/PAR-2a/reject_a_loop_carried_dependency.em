#$ test: compile-fail
#$ rules: PAR-2a, PAR-2
#$ error[E7011]: parallel loop has a loop-carried dependency on `xs`
# `[PAR-2a]` — every access to a place the loop writes, reads included, is at one offset from the
# index: `xs[i + 1]` is read by the iteration after the one that writes it, which may already have
# run on another thread. The help names the ways out.

fn shifts(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len() - 1:
        xs[i] = xs[i + 1]

fn main():
    pass

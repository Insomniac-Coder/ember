#$ test: run-pass
#$ rules: PAR-2a, PAR-2
#$ profiles: debug, release
#$ stdout: [0, 2, 4, 6, 0]
# `[PAR-2a]` — the one-offset rule is about the places the loop writes: one it only reads may be
# reached at any offset (`xs[i - 1]`, `xs[i + 1]`), and the written one is reached only at `i`.

fn main():
    source = [0, 1, 2, 3, 4]
    xs = source.as_mut_span()
    out = [0, 0, 0, 0, 0]
    ys = out.as_mut_span()
    @parallel
    for i in 1..4:
        ys[i] = xs[i - 1] + xs[i + 1]
    println(out)

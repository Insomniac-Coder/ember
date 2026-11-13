# Clause BRW-14.9: "`const ref mut dst: MutSpan[float]` writes the caller's elements and is never
# given another ref, so the caller's `d` is the same span after the call."
#$ test: run-pass
#$ rules: BRW-14, FN-1a, FN-2a
#$ stdout: 5 4

fn fill(const ref mut dst: MutSpan[int], v: int):
    for i in 0..dst.len():
        dst[i] = v

fn main():
    xs = [0, 0]
    d = ref mut xs.as_mut_span()
    fill(const ref mut d, 4)
    d[0] = 5
    println(xs[0], xs[1])

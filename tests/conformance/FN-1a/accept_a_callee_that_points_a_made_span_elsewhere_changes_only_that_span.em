#$ test: run-pass
#$ rules: FN-1a, SPN-3
#$ stdout: [1, 2] [5, 5]
# `[FN-1a]` — through `ref mut` the callee may point the caller's span elsewhere; given a span made
# in the call, it changes only that span, which ends with the line.

fn aim(ref mut dst: MutSpan[int], ref mut other: Array[int]):
    dst = ref mut other.as_mut_span()
    dst[0] = 5
    dst[1] = 5

fn main():
    a = [1, 2]
    b = [0, 0]
    aim(ref mut a.as_mut_span(), ref mut b)
    println(a, b)

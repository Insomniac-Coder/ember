# Clause BRW-14.9: "`const ref mut dst: MutSpan[float]` (...) is never given another ref"
#$ test: compile-fail
#$ rules: BRW-14, BRW-13
# The spec's `const ref mut` message is for a local (`r = const ref mut a`); none is given for a
# parameter: code only.

fn retarget(const ref mut dst: MutSpan[int], ref mut other: Array[int]):
    dst = ref mut other.as_mut_span()    #$ error[E3023]

fn main():
    xs = [0]
    ys = [1]
    d = ref mut xs.as_mut_span()
    retarget(const ref mut d, ref mut ys)

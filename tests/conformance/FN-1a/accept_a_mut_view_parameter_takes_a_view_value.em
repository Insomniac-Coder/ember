#$ test: run-pass
#$ rules: FN-1a, FN-1, SPN-1, SPN-3
# "A `mut` parameter whose declared type is itself a view accepts the view
# value … the mutable-place requirement applies to the place that view was
# taken of, not to the final expression."
#
# This is Part VII §7's own worked example. Read literally, `[FN-1]`'s "the
# argument MUST be a mutable place" rejects it, because `buf.as_mut_span()` is
# a call result rather than a place — and the document would then forbid the
# program it uses to explain itself. ERR-041 recorded the disagreement and the
# owner ruled on 2026-09-10 that the example governs: a mutable borrow derived
# from a mutable place is admitted, an arbitrary value that merely has a
# view type is not.
#
# The compiler already behaved this way (deviation D5), so this case pins
# behaviour rather than changing it — which is why D5 closes as "the compiler
# was right" and no code moved.

fn normalize(mut s: MutSpan[i32]):
    i: usize = 0
    while i < s.len():
        s[i] = s[i] * 2
        i = i + 1

fn main():
    buf: Array[i32] = Array[i32]()
    buf.push(1)
    buf.push(2)
    buf.push(3)
    normalize(buf.as_mut_span())
    println(buf[0])
    println(buf[1])
    println(buf[2])
#$ stdout: 2
#$ 4
#$ 6

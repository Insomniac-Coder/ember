#$ test: run-pass
#$ rules: SPN-1, SPN-2, BRW-2
# A `MutSpan[T]` points into the container rather than at a copy of it, so a
# write through the view is a write to the array. Part VII §7 spells this
# `buf.as_mut_span()`. The read of `a` comes after the view's last use, which
# is what `[BRW-2]` requires.

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    a.push(2)
    s = a.as_mut_span()
    println(s.len())
    s[0] = 40
    println(a[0])
    println(a[1])
#$ stdout: 2
#$ 40
#$ 2

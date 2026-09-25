#$ test: run-pass
#$ rules: STD-11
#$ stdout: 7167
# `[STD-11]` (ODR-033) — insertion is expected constant time, amortised: a
# map held just under its table's limit while keys come and go does not
# rebuild on every other insertion.

fn main():
    m: Map[int, int] = {}
    for i in range(7167):
        m[i] = i
    for i in range(7167, 107167):
        m.remove(i - 7167)
        m[i] = i
    println(len(m))

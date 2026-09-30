#$ test: run-pass
#$ rules: OPT-2, PHIL-5
#$ profiles: debug, release, shipping
#$ stdout:
#$ 503760
#$ 511040
#$ 510800
#$ 14850000
# For MSVC, a loop nest that reads back the list it writes runs in a
# function of its own only when every round adds the same whole number to
# each element (ADR-080): MSVC then adds several rounds at once. The first
# nest moves; the others stay where they are: a round that mixes in its own
# number, a sum kept below 1000 (not one less than a power of two), and an
# addition that keeps its overflow check. The results are the same on every
# compiler.

fn total_of(xs: Array[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn main():
    n = 1000
    a: Array[int] = []
    out: Array[int] = []
    big: Array[int] = []
    for i in 0..n:
        a.push(i % 100)
        out.push(0)
        big.push(0)
    for round in 0..300:
        for i in 0..n:
            out[i] = (out[i] + a[i]) & 1023
    println(total_of(out))

    for round in 0..300:
        for i in 0..n:
            out[i] = ((out[i] ^ round) + a[i]) & 1023
    println(total_of(out))

    for round in 0..300:
        for i in 0..n:
            out[i] = (out[i] + a[i]) & 1000
    println(total_of(out))

    for round in 0..300:
        for i in 0..n:
            big[i] = big[i] + a[i]
    println(total_of(big))

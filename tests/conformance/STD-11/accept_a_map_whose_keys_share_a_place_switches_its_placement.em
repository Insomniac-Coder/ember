#$ test: run-pass
#$ rules: STD-11, HASH-3
#$ profiles: debug, release
#$ stdout: 28655 28672 399980000
# A `Map` puts a key at the remainder of its hash by the table's length, a
# prime, so keys close together get neighbouring places (`spread`: its table
# stays 32,749 places long, a capacity of 28,655). Keys a multiple of that
# length apart all share one place (`piled`): storing them walks further each
# time, and once a store walks past more than 100 used places the map switches
# to the multiply-and-high-bits placement in a power-of-two table, 32,768
# places, a capacity of 28,672, where those keys spread (ADR-100). Every key is
# still found.

fn main():
    spread: Map[int, int] = {}
    for i in 0..20000:
        spread.insert(i * 7, i)
    piled: Map[int, int] = {}
    for i in 0..20000:
        piled.insert(i * 32749, i)
    total = 0
    for i in 0..20000:
        total += spread[i * 7] + piled[i * 32749]
    println(spread.capacity(), piled.capacity(), total)

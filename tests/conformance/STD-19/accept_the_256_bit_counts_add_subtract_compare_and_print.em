#$ test: run-pass
#$ rules: STD-19, TYP-1, TYP-5, TYP-6
#$ stdout: 12 2 true false true
#$ stdout: 340282366920938463463374607431768211455 680564733841876926926749214863536422910 true
#$ stdout: [                                         680564733841876926926749214863536422910]
#$ stdout: 1fffffffffffffffffffffffffffffffe 340282366920938463463374607431768211454 18446744073709551612
#$ stdout: 14
#$ stdout: whole half
#$ stdout: -2 -340282366920938463463374607431768211458 true 3
#$ stdout: -680564733841876926926749214863536422913 115792089237316195423570985008687907852589419931798687112530834793049593217023
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — the 256-bit counts: `u256`, what `len()` gives for a `..=`
# range of 128-bit numbers, and `i256`, signed, what `enumerate` numbers a 128-bit range with (a
# start may be negative). Each adds, subtracts, compares, prints, hashes, and converts to and from
# the other integers with `as`; a narrowing cast takes the low bits, and one between the two
# 256-bit types keeps the bits, as every integer cast does. The narrower unsigned integers widen
# to `u256`, every integer to `i256`.

fn main():
    a = 5 as u256
    b: u256 = 7
    c = a + b
    println(c, b - a, a < b, a == b, c > a)
    big = u128.MAX as u256
    two = big + big
    println(big, two, two - big == big)
    println(f"[{two:>80}]")
    println(f"{two:x}", two as u128, (two - 2) as u64)
    x: u256 = 10
    x += 5
    x -= 1
    println(x)
    counts: Map[u256, str] = Map()
    every = 0 as u128 ..= u128.MAX
    counts[every.iter().len()] = "whole"
    counts[big] = "half"
    println(counts[big + 1], counts[big])
    n = -3
    m: i256 = n
    low = m - (two as i256)
    p = m + 1
    println(p, low + (big as i256), low < m, -m)
    println(low, low as u256)

#$ test: run-pass
#$ rules: STD-26, TYP-1
#$ stdout:
#$ -22 5 [170141183460469231731687303715884105726, 170141183460469231731687303715884105725]
#$ 1
#$ 7 8 0 1000
#$ Some('el') None None Some('ello') [2, 3] [1, 4]
# D-272, `[STD-26]` — `range` and `a..b` over `i128` and `u128` count
# exactly, up to the type's last value, with no step past `stop`. `len`,
# `get` and `drain` take their ranges too; a bound past `int`'s range is out
# of bounds (2^64 + 3 is not 3).

fn main():
    hi: i128 = 170141183460469231731687303715884105727
    total: i128 = 0
    for i in range(hi - 10, hi, 3):
        total = total + (i - hi)
    count = 0
    for i in 0i128..5:
        count = count + 1
    down: Array[i128] = []
    for i in range(hi - 1, hi - 3, -1):
        down.push(i)
    println(total, count, down)
    top: u128 = 340282366920938463463374607431768211455
    last: u128 = 0
    for i in range(top - 5, top, 2):
        last = top - i
    println(last)
    a: i128 = 3
    b: i128 = 10
    lo: i128 = -hi - 1
    println(len(a..b), len(a..=b), len(b..a), len(lo..(lo + 1000)))
    far: u128 = 18446744073709551619
    s = "hello"
    xs: Array[int] = [1, 2, 3, 4]
    taken = xs.drain(1i128..3i128)
    println(s.get(1i128..3i128), s.get(0u128..far), s.get(-1i128..2i128), s.get(1u128..=4u128), taken, xs)

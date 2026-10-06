#$ test: run-pass
#$ rules: TYP-5, TYP-4, STD-19
#$ stdout:
#$ 5 300 16 5 45 200 100
#$ 4 4
# `[TYP-5]` rule 12 (the owner's ruling of 2026-10-06) — a whole number goes into another
# whole-number type, with no `as`, where the compiler knows every value it can have fits:
# by `[STD-19]`'s known numbers (a name set once and never changed, a loop's counter, arithmetic).
# At an assignment, an argument, a return, and in an operator, where the side that fits the other
# side's type converts to it (`int` where both do).

fn takes_a_byte(x: u8) -> int:
    return x as int

fn gives_a_byte() -> u8:
    k = 100
    return k

fn labels_u64(xs: Array[int]) -> u64:
    last: u64 = 0
    for i, _x in enumerate(xs, start=2):
        last = i
    return last

fn main():
    n = 5
    y: u64 = n
    m: u64 = 300
    small: u16 = m
    a: int = 7
    b: u64 = 9
    c = a + b
    w: u128 = 5
    x: int = w
    total: u64 = 0
    for i in 0 .. 10:
        total += i
    big = 200
    println(y, small, c, x, total, takes_a_byte(big), gives_a_byte())
    last: int = 0
    for k, _v in enumerate([10, 20, 30], start=2):
        last = k
    println(labels_u64([10, 20, 30]), last)

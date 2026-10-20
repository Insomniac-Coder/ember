#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, RNG-4
#$ stdout: 49950000 15000
## `[LNT-6]` — `L4003`: an `Array[int]` or `Array[float]` of more than 64 KiB, indexed in a hot loop,
## whose values provably fit 32 bits: every value stored in `codes` is below 1000 (the range facts
## of `[RNG-4]`), and every one stored in `weights` is exactly an `f32`. The loop's counter runs
## past 8191 on every run, so each list holds more than 8192 eight-byte values. A list whose values
## need 64 bits, and a short list, are no lint.

fn main():
    n = 10000
    codes: Array[int] = []
    weights: Array[float] = []
    wide: Array[int] = []
    for i in 0..n:
        codes.push(i % 1000)
        weights.push(1.5)
        wide.push(i * 1000000000)
    short: Array[int] = [1, 2, 3]
    total = 0
    sum = 0.0
    for i in 0..n:
        total += codes[i] * 10 + wide[i] % 7 - wide[i] % 7  #$ warning[L4003]: `codes` holds more than 64 KiB of `i64` values that all fit in `i32`, and this hot loop indexes it
        sum += weights[i]  #$ warning[L4003]: `weights` holds more than 64 KiB of `f64` values that all fit in `f32`, and this hot loop indexes it
    for j in 0..3:
        total += short[j] - short[j]
    println(total, sum as int)

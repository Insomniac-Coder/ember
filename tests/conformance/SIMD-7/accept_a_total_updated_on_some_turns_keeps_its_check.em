#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, release, shipping
#$ stdout:
#$ 12000 2000
#$ 2139
#$ assert-c: !contains(" >> 56ULL;")
# A running total updated on only some turns (`if b == 111: count += 1`)
# keeps its check on each update: its loop has a branch, so the C compiler
# does not vectorise blocks of it, and the blocks would only add their tests
# (D-499: `a03_strings` 1.27x with MSVC). An indexed total is the same.

fn main():
    s: String = ""
    for i in 0..1000:
        s.push_str("hello ")
        s.push_str("world ")
    count = 0
    for b in s.as_bytes():
        if b == 111:
            count = count + 1
    println(len(s.as_bytes()), count)
    xs: Array[int] = []
    for i in 0..1000:
        xs.push(i % 7)
    total = 0
    for i in 0..1000:
        if xs[i] > 3:
            total = total + xs[i]
    println(total)

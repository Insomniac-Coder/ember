#$ test: run-pass
#$ rules: STD-26, CTL-3
#$ stdout:
#$ Range(start=0, end=5)
#$ 2 10 true
#$ 0 1 2 | 0 1 2 |
# `[STD-26]` — `range(n)` is `0..n` and `range(a, b)` is `a..b`, as values too:
# a `Range` that can be kept, measured, tested and iterated (ODR-027). The
# start of `range(n)` takes `n`'s type.

r = range(5)
println(r)
s = range(2, 4)
println(len(s), len(range(10)), 3 in range(1, 4))
n: u8 = 3
u = range(n)
for i in u:
    print(i, end=" ")
print("| ")
for i in u:
    print(i, end=" ")
println("|")

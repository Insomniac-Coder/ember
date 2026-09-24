#$ test: run-pass
#$ rules: CTL-3, CTL-3b
#$ stdout:
#$ 0 1 2 3 | 0 1 2 3 |
#$ 9 6
#$ 0 4 0 4
#$ -2 -1 0 1 2 |
#$ 250 251 252 | 7 8 9 | 2
#$ 5 true false
#$ Range(start=0, end=4)
# `[CTL-3]` (ODR-027) — `a..b`, `a..=b`, `a..` and `..b` are values of the
# prelude's `Range`, `RangeInclusive`, `RangeFrom` and `RangeTo`: `Copy`, with
# public bounds. A `for` over one counts over a copy of its bounds, so the
# range can be iterated again; an expected range type gives the bound's type.

fn total(r: Range[int]) -> int:
    t = 0
    for i in r:
        t += i
    return t

fn small(n: u8) -> RangeInclusive[i16]:
    return -2..=n as i16

r = 0..4
for i in r:
    print(i, end=" ")
print("| ")
for i in r:
    print(i, end=" ")
println("|")
println(total(2..5), total(r))
s = r
println(s.start, s.end, r.start, r.end)
for x in small(2):
    print(x, end=" ")
println("|")
for i in 250u8..:
    if i == 253:
        break
    print(i, end=" ")
print("| ")
from_seven = 7..
for i in from_seven:
    if i > 9:
        break
    print(i, end=" ")
print("| ")
k = 0
for i in 0..:
    k += 1
    if k < 3:
        continue
    println(i)
    break
upto = ..5
println(upto.end, r == 0..4, r == 0..5)
println(r)

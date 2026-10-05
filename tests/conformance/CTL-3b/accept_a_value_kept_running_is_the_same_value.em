#$ test: run-pass
#$ rules: CTL-3b, TYP-10
#$ profiles: debug, release, shipping
#$ stdout:
#$ 9 49 121 225 361 529 729 961 1225 1521
#$ 15:25 22:144 29:361 36:676
#$ 67120
#$ 1,0,0 4,1,9 9,2,36 16,3,81 25,4,144 36,5,225 49,6,324 64,7,441 81,8,576 100,9,729 121,10,900 144,11,1089 169,12,1296 196,13,1521
#$ -8088427885084532062 -8709353554835271723
# A value a counted loop computes from its counter (`k * step + skip`, an
# `enumerate` number, a stepped range's value, `i * c + d` under
# `@overflow(wrap)`) may be kept as a running value, advanced in the loop's
# step, when the C is for a compiler that vectorises one (clang, gcc): the
# results are the same with `continue`, `break`, nested loops, `a..=b`, a
# value read after the loop and arithmetic that wraps.

@overflow(wrap)
fn mixed(n: int) -> int:
    acc = 0
    last = 0
    for i in 0..n:
        if i & 7 == 3:
            continue
        acc ^= i * 7046029254386353131 + 7
        last = 100 - i * 2
        if i == 900:
            break
    for i in 0..=n // 10:
        acc ^= i * 3 + 1
    return acc ^ last

fn main():
    xs: Array[int] = []
    for i in 0..40:
        xs.push(i * i)
    line: Array[String] = []
    for x in xs.iter().skip(3).step_by(4):
        if x % 2 == 0:
            continue
        line.push(f"{x}")
    println(line.join(" "))
    line.clear()
    for (i, x) in xs.iter().enumerate(start=10).skip(5).step_by(7):
        line.push(f"{i}:{x}")
        if i > 30:
            break
    println(line.join(" "))
    line.clear()
    total = 0
    for round in 0..4:
        # G8-4 — a range from a variable numbers in `i128` (the visible-numbers rule sees `round`'s
        # type, not its values), so the number is converted where it is added to an `int`.
        for (i, v) in (round..30).step_by(round + 1).enumerate(start=round):
            total += (i as int) * 100 + v
    println(total)
    for (a, (b, c)) in xs.iter().skip(1).zip(xs.iter().step_by(3).enumerate()):
        line.push(f"{a},{b},{c}")
    println(line.join(" "))
    println(mixed(1000), mixed(7))

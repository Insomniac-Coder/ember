#$ test: run-pass
#$ rules: RNG-4, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 56372 1022 49500
#$ assert-c-count: contains("ember_ck_") == 0
# `[RNG-4]` — a list this function makes empty and changes only by `push`
# and element writes holds only values it stored: `a` holds `i % 1000`
# (0..999), `out` holds 0 and `... & 1023` (0..1023). So `out[i] ^ round`
# is below 32768 (`round`, the outer counter, keeps its range inside the
# inner loop), and adding `a[i]` and taking 7 cannot overflow. Read through
# a view (`for x in out`), `x + 1` cannot either; the totals run 100 turns
# of bounded values. No check is left.

fn main():
    n = 100
    a: Array[int] = []
    out: Array[int] = []
    for i in 0..n:
        a.push(i % 1000)
        out.push(0)
    for round in 0..50:
        for i in 0..n:
            out[i] = (((out[i] ^ round) + a[i]) - 7) & 1023
    total = 0
    for i in 0..n:
        total = total + out[i]
    peak = 0
    for x in out:
        y = x + 1
        if y > peak:
            peak = y
    sum_a = 0
    for i in 0..n:
        sum_a = sum_a + a[i] * 10
    println(total, peak, sum_a)

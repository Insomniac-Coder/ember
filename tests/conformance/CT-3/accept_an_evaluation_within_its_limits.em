#$ test: run-pass
#$ rules: CT-3, CT-1
#$ stdout: 499999500000 1000000
# `[CT-3]` — each evaluation may take up to 10⁸ steps (a function entered or a loop turned) and
# 256 MB of evaluator heap by default; these stay well within both.

fn sum_to(n: int) -> int:
    total = 0
    for i in range(n):
        total += i
    return total

fn filled(n: int) -> int:
    xs: Array[int] = []
    for i in range(n):
        xs.push(i)
    return xs.len()

fn main():
    println(comptime(sum_to(1_000_000)), comptime(filled(1_000_000)))

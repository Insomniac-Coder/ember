#$ test: run-pass
#$ rules: TYP-4, TYP-5, EXP-3, CTL-4

fn max(a: i32, b: i32) -> i32:
    if a > b:
        return a
    return b

fn sum_to(n: i32) -> i32:
    total = 0
    i = 1
    while i <= n:
        total += i
        i += 1
    return total

fn main():
    println(max(3, 9))
    println(sum_to(10))
    println(2 * 3 + 4)
    println(1.5 + 2.25)
    println(true and false)
    println(not (1 == 2))
#$ stdout: 9
#$ 55
#$ 10
#$ 3.75
#$ false
#$ true

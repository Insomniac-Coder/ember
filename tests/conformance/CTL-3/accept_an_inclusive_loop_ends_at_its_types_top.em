#$ test: run-pass
#$ rules: CTL-3, CTL-4, OPT-2
#$ profiles: debug, release
#$ stdout:
#$ 18446744073709551613 18446744073709551614 18446744073709551615 |
#$ 9223372036854775806 9223372036854775807 | 253 254 255 | 2147483646 2147483647 |
#$ 6 10 0 | 5 10 0
#$ 9223372036854775805 9223372036854775807 else
#$ 100 1 3
# `[CTL-3]` — `a..=MAX` terminates after `MAX`. Until D-525 the counter
# stepped past the top and the loop ran forever, the range facts having
# removed the body's own `break`. A loop whose end is not known to be below
# the top (`sum_to`, `count_to`) runs a copy without the test for its last
# turn when the end is below the top; `continue` and `else` keep their
# meaning on the last turn.

fn sum_to(a: u64, b: u64) -> u64:
    total: u64 = 0
    for i in a..=b:
        total += i - a
    return total

fn count_to(a: int, b: int) -> int:
    n = 0
    for _ in a..=b:
        n += 1
    return n

fn main():
    turns = 0
    for i in (u64.MAX - 2) ..= u64.MAX:
        print(i, end=" ")
        turns += 1
        if turns > 5:
            print("ran past the top", end=" ")
            break
    println("|")
    for i in (int.MAX - 1) ..= int.MAX:
        print(i, end=" ")
    print("| ")
    for i in 253u8 ..= u8.MAX:
        print(i, end=" ")
    print("| ")
    for i in (i32.MAX - 1) ..= i32.MAX:
        print(i, end=" ")
    println("|")
    print(sum_to(u64.MAX - 3, u64.MAX), sum_to(5, 9), sum_to(9, 5), "| ")
    println(count_to(int.MAX - 4, int.MAX), count_to(1, 10), count_to(3, 2))
    for i in (int.MAX - 2) ..= int.MAX:
        if i == int.MAX - 1:
            continue
        print(i, end=" ")
    else:
        println("else")
    xs: Array[int] = [10, 20, 30, 40]
    total = 0
    last = xs.len() - 1
    for i in 0..=last:
        total += xs[i]
    kept: Array[int] = []
    for i in 1..=3:
        if i == 2:
            continue
        kept.push(i)
    println(total, kept[0], kept[1])

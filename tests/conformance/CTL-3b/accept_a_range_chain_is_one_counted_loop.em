#$ test: run-pass
#$ rules: CTL-3b, STD-19
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("_next(") == 0
#$ stdout:
#$ 0=3 1=6 2=9
#$ 250 251 252 253 254 255
#$ 9223372036854775806 9223372036854775807
#$ 4 7 | 2 5
#$ -9223372036854775807 0 9223372036854775807
# `[CTL-3b]` — a range is `Iterable` (`r.iter()`, and the adapters on it), and
# a chain over a range's iterator is one counted loop too, each value computed
# from its index. `a..=b` ending at its type's top never steps past it, and
# one spanning all of `int` (2^64 values, more than a 64-bit count holds) is
# counted by its last index.

fn main():
    line: Array[String] = []
    for (i, v) in (3..=9).step_by(3).enumerate():
        line.push(f"{i}={v}")
    println(line.join(" "))
    line.clear()
    for v in (250u8..=255u8).iter():
        line.push(f"{v}")
    println(line.join(" "))
    line.clear()
    for v in (9223372036854775805..=9223372036854775807).skip(1):
        line.push(f"{v}")
    println(line.join(" "))
    line.clear()
    n = 9
    for v in (2..n).skip(2).step_by(3):
        line.push(f"{v}")
    line.push("|")
    for v in (2..n).take(4).step_by(3):
        line.push(f"{v}")
    println(line.join(" "))
    line.clear()
    top = 9223372036854775807
    bottom = -top - 1
    for v in (bottom..=top).skip(1).step_by(top):
        line.push(f"{v}")
    println(line.join(" "))
    line.clear()

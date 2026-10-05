#$ test: run-pass
#$ rules: STD-19, CTL-3b
#$ stdout: [2, 1, 0]
#$ stdout: [18446744073709551614, 18446744073709551613, 18446744073709551612]
#$ stdout: [18446744073709551614, 18446744073709551612, 18446744073709551610]
#$ stdout: [(18446744073709551611, 18446744073709551614), (18446744073709551610, 18446744073709551613)]
#$ stdout: [(18446744073709551614, 18446744073709551614), (18446744073709551613, 18446744073709551613)]
#$ stdout: 18446744073709551615
#$ stdout: 2f 18446744073709551614
#$ stdout: 3f 18446744073709551614
#$ stdout: [340282366920938463463374607431768211455, 340282366920938463463374607431768211454]
#$ stdout: [-9223372036854775807, -9223372036854775808]
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — the six cases of question 8 on a range too long for an `int`
# to count, `0 as u64 .. u64.MAX`: `take`, `skip`, `step_by`, `enumerate` and `zip` run backwards
# and `len()` answers, step by step through std's adapters and as `for` loops. Each panicked
# before ("a range of more than int.MAX values has no length", or an overflow).

fn first[I: Iterator](owned from: I, n: int) -> Array[I.Item]:
    it = from
    out: Array[I.Item] = []
    while out.len() < n:
        match it.next():
            Some(x):
                out.push(x)
            None:
                return out
    return out

fn main():
    big = 0 as u64 .. u64.MAX
    println(first(big.iter().take(3).rev(), 3))
    println(first(big.iter().skip(5).rev(), 3))
    println(first(big.iter().step_by(2).rev(), 3))
    println(first(big.iter().enumerate(-3).rev(), 2))
    println(first(big.iter().zip(big.iter()).rev(), 2))
    println(big.iter().len())
    for x in big.iter().skip(5).rev():
        println("2f", x)
        break
    for x in big.iter().step_by(2).rev():
        println("3f", x)
        break
    every = 0 as u128 ..= u128.MAX
    println(first(every.iter().skip(1).rev(), 2))
    whole = int.MIN ..= int.MAX
    println(first(whole.iter().take(2).rev(), 2))

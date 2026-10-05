#$ test: run-pass
#$ rules: STD-19
#$ stdout: [(2, 2), (1, 1), (0, 0)]
#$ stdout: [18446744073709551613, 18446744073709551614]
#$ stdout: [18446744073709551610, 18446744073709551611, 18446744073709551612, 18446744073709551613, 18446744073709551614]
#$ stdout: Some(9223372036854775807)
#$ stdout: [(18446744073709551613, 18446744073709551608), (18446744073709551614, 18446744073709551609)]
#$ stdout: [(1, 9223372036854775805), (0, 9223372036854775806)]
#$ profiles: debug, release
# G8-4 (the owner's design, 0.9.10) — no adapter passes a gigantic run of items one at a time:
# leaving items out (`skip_front`, `skip_back`) is done at once by a range and passed down by
# `rev`, `take`, `skip`, `zip`, `copied`, `chain`, and `enumerate` from the back, so `take`,
# `skip`, `nth` and `rev` over the whole `0 as u64 .. u64.MAX`, a range of 18 billion billion
# values, finish at once. Each of these stepped through the range one value at a time.

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
    println(first(big.iter().enumerate().take(3).rev(), 3))
    println(first(big.iter().rev().take(2).rev(), 2))
    println(first(big.iter().skip(int.MAX).skip(9223372036854775803), 5))
    println(big.iter().nth(9223372036854775807))
    println(first(big.iter().skip(5).zip(big.iter()).rev().take(2).rev(), 2))
    whole = int.MIN .. int.MAX
    println(first(whole.iter().rev().enumerate().take(2).rev(), 2))

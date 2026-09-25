#$ test: run-pass
#$ rules: STD-11, HASH-1, TYP-1
#$ stdout:
#$ low high 2
#$ true false
# D-272 — `i128` and `u128` hash (std writes their `Hash` as two `u64`s), so
# they key a `Map` and fill a `Set`.

fn main():
    hi: i128 = 170141183460469231731687303715884105727
    lo: i128 = -hi - 1
    m: Map[i128, String] = {lo: String.from("low"), hi: String.from("high")}
    println(m[lo], m[hi], len(m))
    one: u128 = 1
    s: Set[u128] = {one << 64, one}
    println(one << 64 in s, 2u128 in s)

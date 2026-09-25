#$ test: run-pass
#$ rules: TYP-38, GRM-26
#$ stdout: {'ann': 32, 'bob': 27} 2
#$ stdout: {} 0
#$ stdout: {2, 3, 5} 3
#$ stdout: {1: [1, 2], 2: []}
#$ stdout: {'k': 1, 'j': 2}
# `[TYP-38]`, `[GRM-26]` (ODR-034, ODR-036) — `{k: v}` is a `Map`, `{a}` a
# `Set` and `{}` an empty map. A repeated key keeps its first position and
# its last value; text with no context is `String`. A literal may span lines.

fn main():
    ages = {"ann": 31, "bob": 27, "ann": 32}
    println(ages, len(ages))
    empty: Map[String, int] = {}
    println(empty, len(empty))
    primes = {2, 3, 5, 3}
    println(primes, len(primes))
    nested = {1: [1, 2], 2: []}
    println(nested)
    multi = {
        "k": 1,
        "j": 2,
    }
    println(multi)

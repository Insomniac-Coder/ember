#$ test: compile-fail
#$ rules: DIA-14, DIA-2
# `[DIA-14]` (D-822) — one error for one bad expression: the failed `" ".join(row)` is reported,
# and the `+`s around it say nothing more; a `match` on a failed index, and a bound on a tuple of
# failed indices, say nothing either, so no message names an error type.

fn main():
    row = ["a", "b"]
    s = "[" + " ".join(row) + " ]"    #$ error[E2072]: `str` has no method named `join`
    println(s)
    parts = "a b".split(" ")
    match parts[0]:    #$ error[E2020]: cannot index `std.string.Split`
        "a" => println(1)
        _ => println(2)
    pairs: Array[(String, int)] = [("a", 1)]
    pairs.sort_by_key(fn(p) => (p[1], 0))    #$ error[E2020]: cannot index `(String, i64)`

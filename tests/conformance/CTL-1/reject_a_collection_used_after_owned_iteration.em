#$ test: compile-fail
#$ rules: CTL-1, EXP-6
# `[CTL-1]`, `[EXP-6]` — `for k in owned m:` moves `m` into the loop; it is
# not there afterwards.

fn main():
    m: Map[String, i64] = {"a": 1}
    for k in owns m:
        println(k)
    println(m) #$ error[E3040]: `m` was handed over to `into_iter` on line 8 and is used on line 10; to keep both, hand over a copy of your own: `copy of m`

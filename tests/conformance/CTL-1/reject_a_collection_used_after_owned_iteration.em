#$ test: compile-fail
#$ rules: CTL-1, EXP-6
# `[CTL-1]`, `[EXP-6]` — `for k in owned m:` moves `m` into the loop; it is
# not there afterwards.

fn main():
    m: Map[String, i64] = {"a": 1}
    for k in owned m:
        println(k)
    println(m) #$ error[E3050]: `m` is borrowed after it has been moved out of

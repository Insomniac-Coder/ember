# Clause BRW-13.2: "the name now views something else (...), checked by `[BRW-1]` against its new
# target."
#$ test: compile-fail
#$ rules: BRW-13, BRW-1
# The spec gives no message text for this case: code only.

fn main():
    a = 1
    b = 2
    v = ref a
    println(v)
    v = ref b
    b = 6    #$ error[E3021]
    println(v)

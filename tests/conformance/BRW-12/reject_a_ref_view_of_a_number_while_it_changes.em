# Clause BRW-12.10: "`ref` ... needed before a part only to view a number (`r = ref counts[0]`)" --
# `r` views the element, so it is not changed while `r` is used (`[BRW-1]`).
#$ test: compile-fail
#$ rules: BRW-12, BRW-1
# The spec gives no message text for this case: code only.

fn main():
    counts = [1, 2]
    r = ref counts[0]
    counts[0] = 5    #$ error[E3021]
    println(r)

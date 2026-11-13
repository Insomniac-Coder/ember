# Clause BRW-13.4: "The line that gives it another ref says `ref mut` again, for every type, and a
# read-only `ref b` given to it is `E3023` ("`r` may change what it refers to, and `ref b` only reads
# `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`")."
#$ test: compile-fail
#$ rules: BRW-13, TYP-14
# The first message is the spec's. The second, for a `String`, follows Q4's ruling (INDEX.md): the
# new name is the refused name followed by the target's root name (`rb`, `tsecond`).

fn main():
    a = 1
    b = 2
    r = ref mut a
    r = ref b    #$ error[E3023]: `r` may change what it refers to, and `ref b` only reads `b`; to point `r` at `b` write `r = ref mut b`, or for a read-only name `rb = ref b`
    println(r, b)
    first = String("ab")
    second = String("cd")
    t = ref mut first
    t = ref second    #$ error[E3023]: `t` may change what it refers to, and `ref second` only reads `second`; to point `t` at `second` write `t = ref mut second`, or for a read-only name `tsecond = ref second`
    println(t, second)

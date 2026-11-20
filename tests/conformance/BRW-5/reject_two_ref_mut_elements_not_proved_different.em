# Clause BRW-5.1: "`ref mut a[i]` and `ref mut a[j]`, as two names or as two arguments of one call,
# are `E3022` unless the compiler proves `i` and `j` differ: both constants and different, `j` from
# `i + 1..`, or the line inside `if i != j:`. Nothing is compared while the program runs. With `fn
# collide(ref mut a: Body, ref mut b: Body):`, `collide(ref mut bodies[i], ref mut bodies[j])` in a
# function given `i` and `j` is refused ("`collide` changes `a` and `b` as two different bodies, and
# `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and
# indent it under it")"
#$ test: compile-fail
#$ rules: BRW-5, BRW-11, BCK-3, BCK-5
# The call's message is the spec's. Two locals with constant values are not constants (the list of
# proofs is closed: `decisions.md` §4, area 2 Q2). Codes only for the two names.

struct Body:
    v: int

fn collide(ref mut a: Body, ref mut b: Body):
    a.v += b.v
    b.v = 0

fn as_arguments(i: int, j: int):
    bodies = [Body(v=1), Body(v=2)]
    collide(ref mut bodies[i], ref mut bodies[j])    #$ error[E3022]: `collide` changes `a` and `b` as two different bodies, and `bodies[i]` and `bodies[j]` are the same body when `i == j`; write `if i != j:` above this line and indent it under it

fn as_names(i: int, j: int):
    bodies = [Body(v=1), Body(v=2)]
    a = ref mut bodies[i]
    b = ref mut bodies[j]    #$ error[E3022]
    a.v += b.v

fn from_locals():
    bodies = [Body(v=1), Body(v=2)]
    i = 0
    j = 1
    a = ref mut bodies[i]
    b = ref mut bodies[j]    #$ error[E3022]
    a.v += b.v

fn main():
    as_arguments(0, 1)
    as_names(0, 1)
    from_locals()

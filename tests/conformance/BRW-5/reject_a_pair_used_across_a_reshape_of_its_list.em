# Clause BRW-5.4: "The pair points into the list it was called on (`[LT-1]` rule 1)."
#$ test: compile-fail
#$ rules: BRW-5, LT-1, BRW-1, BRW-17
# The two views are `ref mut` addresses into `xs` (views a call gives): a reshape of `xs` while they
# are used is refused at the reshape (`E3022`, `[BRW-1]`). Codes only.

fn main():
    xs = [1, 2, 3]
    if xs.get_pair_mut(0, 1) exists as (ref mut a, ref mut b):
        xs.push(owns 4)    #$ error[E3022]
        a += b

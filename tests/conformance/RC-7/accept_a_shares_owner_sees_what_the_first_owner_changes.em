#$ test: run-pass
#$ rules: RC-7, RC-1
#$ stdout: 4
## Design test `run_a_shares_owner_sees_what_the_first_owner_changes.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: a `shares` owner only reads the value, and "sees changes made through the value's other owners
## that may change it" (R8 (g)); the name it was shared from keeps what it could do (AN-34's
## `owner_that_reads`, verdict 23).

fn main():
    items = [1, 2, 3]
    other = shares items          # one list, two owners; `other` only reads it
    items.push(owns 4)                 # the first owner still changes it
    println(other.len())          # 4

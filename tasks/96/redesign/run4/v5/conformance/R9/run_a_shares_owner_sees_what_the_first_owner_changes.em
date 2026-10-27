#$ test: run-pass
#$ rules: RC-1, OWN-8, BRW-16
#$ stdout: 4
## design rules: R9, R3, R8
## R9: a `shares` owner only reads the value, and "sees changes made through the value's other owners
## that may change it" (R8 (g)); the name it was shared from keeps what it could do (AN-34's
## `owner_that_reads`, verdict 23).

fn main():
    items = [1, 2, 3]
    other = shares items          # one list, two owners; `other` only reads it
    items.push(4)                 # the first owner still changes it
    println(other.len())          # 4

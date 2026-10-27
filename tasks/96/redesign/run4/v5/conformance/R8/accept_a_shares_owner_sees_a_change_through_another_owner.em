#$ test: run-pass
#$ rules: BRW-16, RC-1, OWN-8
#$ stdout: 4
## design rules: R8, R9, R3
## R8 (g): a `shares` owner is an owner, not a view: it sees a change made through the value's other
## owners that may change it (R9; verdict 23: the first name keeps what it could do).

fn main():
    items = [1, 2, 3]
    other = shares items
    items.push(4)
    println(other.len())

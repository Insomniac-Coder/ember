#$ test: run-pass
#$ rules: R8, R9, R3
#$ stdout: 4
## R8 (g): a `shares` owner is an owner, not a view: it sees a change made through the value's other
## owners that may change it (R9; verdict 23: the first name keeps what it could do). Once shared,
## `items` is one of two owners, a link (section 0), so `main`, which pushes through it, says
## `changes Array[int]` (R5, verdict 64).

fn main() changes Array[int]:
    items = [1, 2, 3]
    other = shares items
    items.push(4)
    println(other.len())

#$ test: compile-fail
#$ rules: R1
#$ help: to keep both, write `snapshot = copy of items` (a copy of your own) or `snapshot = shares items` (two owners of one list)
## R1: `b = a`, where `a` is a whole name that owns a value whose type is not `Copy`, hands `a` over:
## `a` ends, and a later use of it is refused (E3040), naming both ways out, `copy of` and `shares`
## (row 9; AN-34's refused program, with the message of `V3/rules.md` section 3).

fn main():
    items = [1, 2, 3]
    snapshot = items
    items.push(4)              #$ error[E3040]: `items` was handed over to `snapshot` on line 10 and is used on line 11
    println(snapshot.len())

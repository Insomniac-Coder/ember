#$ test: run-pass
#$ rules: R3, R9
#$ stdout: 4
## R3, R9: `shares mut items` makes one more owner that may change the list; a change through either
## is seen by both (AN-34's `two_owners`). A change through a `shares mut` owner is a change through a
## link, named on the first line whatever holds it, the function's own local included (verdict 64).

fn main() changes Array[int]:
    items = [1, 2, 3]
    other = shares mut items
    other.push(4)
    println(items.len())

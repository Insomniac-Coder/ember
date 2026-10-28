#$ test: run-pass
#$ rules: OWN-8, RC-1
#$ stdout: 4
## design rules: R3, R9
## R3, R9: `shares mut items` makes one more owner that may change the list; a change through either
## is seen by both (AN-34's `two_owners`). No first line names the change (verdict 86).

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(4)
    println(items.len())

#$ test: compile-fail
#$ rules: R3, R9
#$ help: to change it through `other`, write `other = shares mut items`
## R3, R9: `shares x` makes one more owner that reads it; a change through it is refused (E3023;
## AN-34's message). `main` names the change it means to make through a link (verdict 64), so the
## help's line compiles (verdict 73).

fn main() changes Array[int]:
    items = [1, 2, 3]
    other = shares items
    other.push(4)              #$ error[E3023]: `other.push(4)` changes the list, but `other` only reads the list it shares
    println(items.len())

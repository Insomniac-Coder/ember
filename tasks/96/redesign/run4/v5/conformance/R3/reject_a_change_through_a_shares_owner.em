#$ test: compile-fail
#$ rules: BRW-14, BRW-15
#$ help: to change it through `other`, write `other = shares mut items`
## design rules: R3, R9
## R3, R9: `shares x` makes one more owner that reads it; a change through it is refused (E3023;
## AN-34's message). The help's line compiles as it is: no first line names a change (verdict 86).

fn main():
    items = [1, 2, 3]
    other = shares items
    other.push(4)              #$ error[E3023]: `other.push(4)` changes the list, but `other` only reads the list it shares
    println(items.len())

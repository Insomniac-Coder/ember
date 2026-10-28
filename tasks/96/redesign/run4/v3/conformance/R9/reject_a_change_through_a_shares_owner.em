#$ test: compile-fail
#$ rules: R9, R3, R5
#$ help: to change it through `other`, write `other = shares mut items`
## R9: "a `shares` owner only reads it"; only an owner made with `shares mut`, and the name the value
## was first shared from, change it (AN-34, still refused; the `shares` row of `V3/vocabulary.md`).

fn main():
    items = [1, 2, 3]
    other = shares items
    other.push(4)                 #$ error[E3023]: `other.push(4)` changes the list, but `other` only reads the list it shares
    println(items.len())

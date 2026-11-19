#$ test: compile-fail
#$ rules: RC-7, RC-1
#$ help: to change it through `other`, write `other = shares mut items` in place of line 9
# `[RC-7]` — a `shares` owner only reads the value it shares (`E3023`); only an owner made with
# `shares mut`, and the name the value was first shared from, change it.

fn main():
    items = [1, 2, 3]
    other = shares items
    other.push(owns 4)                #$ error[E3023]: `other.push(owns 4)` changes the list, but `other` only reads the list it shares
    println(items.len())

#$ test: compile-pass
#$ rules: R9, R3, R5
## R9: an owner made with `shares mut` may change the shared value (the nearest allowed program to
## `reject_a_change_through_a_shares_owner.em`, AN-34). R5 (verdict 64): "a change through a link always
## says `changes T`, whatever holds the link", the function's own local included, so `main` names the list.

fn main() changes Array[int]:
    items = [1, 2, 3]
    other = shares mut items
    other.push(4)
    println(items.len())

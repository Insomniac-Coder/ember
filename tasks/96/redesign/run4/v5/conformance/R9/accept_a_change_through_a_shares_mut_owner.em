#$ test: compile-pass
#$ rules: BRW-15
## design rules: R9, R3, R5
## R9: an owner made with `shares mut` may change the shared value (the nearest allowed program to
## `reject_a_change_through_a_shares_owner.em`, AN-34).

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(4)
    println(items.len())

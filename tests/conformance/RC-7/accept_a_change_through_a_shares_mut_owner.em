#$ test: compile-pass
#$ rules: RC-7, RC-1, OWN-13
## Design test `accept_a_change_through_a_shares_mut_owner.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: an owner made with `shares mut` may change the shared value (the nearest allowed program to
## `reject_a_change_through_a_shares_owner.em`, AN-34).

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(owns 4)
    println(items.len())

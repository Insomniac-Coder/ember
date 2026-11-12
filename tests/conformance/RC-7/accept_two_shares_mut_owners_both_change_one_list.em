#$ test: run-pass
#$ rules: RC-7, RC-1
#$ stdout: 4
#$ stdout: 5
## Design test `run_two_shares_mut_owners_both_change_one_list.em` of `tasks/96/redesign/run4/v3/conformance/`; where it says so, `owns` is written at an `owns` parameter (`[FN-2a]`) and a `changes` list on a method's first line is left out.
## R9: "after `other = shares mut items` both `items` and `other` may change the list"; each owner sees
## the other's change, and nothing is checked while the program runs (AN-34's `two_owners`, verdict 23).

fn main():
    items = [1, 2, 3]
    other = shares mut items      # one list, two owners, both may change it
    other.push(owns 4)
    println(items.len())          # 4: the change through `other` is seen through `items`
    items.push(owns 5)                 # the first owner keeps what it could do
    println(other.len())          # 5

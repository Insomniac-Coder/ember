#$ test: run-pass
#$ rules: R9, R3
#$ stdout: 4
#$ stdout: 5
## R9: "after `other = shares mut items` both `items` and `other` may change the list"; each owner sees
## the other's change, and nothing is checked while the program runs (AN-34's `two_owners`, verdict 23).

fn main():
    items = [1, 2, 3]
    other = shares mut items      # one list, two owners, both may change it
    other.push(4)
    println(items.len())          # 4: the change through `other` is seen through `items`
    items.push(5)                 # the first owner keeps what it could do
    println(other.len())          # 5

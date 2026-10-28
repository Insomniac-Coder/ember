#$ test: run-pass
#$ rules: OWN-8, RC-1
#$ stdout: 4
## design rules: R3, R9
## R3, R9: the name a value is shared from keeps what it could do: after `other = shares items`,
## `items` may still change the list, and the reading owner sees it (verdict 23; AN-34's
## `owner_that_reads`).

fn main():
    items = [1, 2, 3]
    other = shares items
    items.push(4)
    println(other.len())

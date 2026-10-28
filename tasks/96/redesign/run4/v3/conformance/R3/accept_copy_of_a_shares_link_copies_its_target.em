#$ test: run-pass
#$ rules: R3
#$ stdout: 2 2 3
## R3: before a `shares` link, `copy of` copies its target, never the link: `mine` is a list of its
## own, and a push to it is not seen through `items` or `other`.

fn main():
    items = [1, 2]
    other = shares mut items
    mine = copy of other
    mine.push(3)
    println(items.len(), other.len(), mine.len())

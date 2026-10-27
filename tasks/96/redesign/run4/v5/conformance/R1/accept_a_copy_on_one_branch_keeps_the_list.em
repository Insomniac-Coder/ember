#$ test: run-pass
#$ rules: OWN-1, OWN-8
#$ stdout: 3 3
## design rules: R1, R3
## R1, R3: the nearest program that is allowed: the branch takes a copy of its own, so `items` is
## whole on every path.

fn main():
    items = [1, 2, 3]
    keep = true
    kept: Array[int] = []
    if keep:
        kept = copy of items
    println(items.len(), kept.len())

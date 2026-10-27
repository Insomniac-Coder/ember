#$ test: run-pass
#$ rules: R1, R3
#$ stdout: 3 4
## R1, R3: the first way out of row 9: `copy of` makes a list of your own, so the push to `items`
## is not seen in `snapshot` (AN-34's NEW `main`).

fn main():
    items = [1, 2, 3]
    snapshot = copy of items
    items.push(4)
    println(snapshot.len(), items.len())

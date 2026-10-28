#$ test: run-pass
#$ rules: R1
#$ stdout: 4
## R1: after `snapshot = items` the list is `snapshot`'s; using it through `snapshot` is the use the
## hand-over allows (AN-34's `hand_over`).

fn main():
    items = [1, 2, 3]
    snapshot = items
    snapshot.push(4)
    println(snapshot.len())

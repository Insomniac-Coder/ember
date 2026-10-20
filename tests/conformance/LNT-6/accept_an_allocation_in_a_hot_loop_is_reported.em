#$ test: run-pass
#$ command: lint
#$ rules: LNT-6
#$ stdout: 9 3
## `[LNT-6]` — `L4001`: a new value owning heap memory made on every turn of a hot loop, an
## innermost one (ADR-241). Growing a list made before the loop, and an allocation in a loop that
## holds another, are no lint.

fn main():
    names: Array[String] = []
    lengths: Array[int] = []
    for i in 0..3:
        label = String.from("item")  #$ warning[L4001]: allocation in a hot loop
        lengths.push(label.len() + i)
    for round in 0..3:
        names.push(String.from("x"))
        for j in 0..2:
            lengths.push(j)
    println(lengths.len(), names.len())

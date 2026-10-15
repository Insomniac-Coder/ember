#$ test: compile-fail
#$ rules: SOA-2, BRW-1
#$ error[E3022]: `ns.col_score` is already mutably borrowed
# `[SOA-2]` — a length-changing operation borrows all of the `SoA`, so it conflicts with a column
# borrowed before it and used after.

struct Named:
    name: String
    score: int

fn main():
    ns = SoA[Named].with_capacity(2)
    ns.push(Named("ann".to_string(), 3))
    scores = ns.score
    ns.push(Named("dee".to_string(), 1))
    println(scores.len())

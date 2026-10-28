#$ test: run-pass
#$ rules: R13
#$ stdout: 5
## R13: the nearest program that is allowed for both E0115 tests: `Some(...)` makes a value
## (`best = Some(5)`) and names a `match` branch.

fn main():
    best: Option[int] = None
    best = Some(5)
    match best:
        Some(v):
            println(v)
        None:
            println("none")

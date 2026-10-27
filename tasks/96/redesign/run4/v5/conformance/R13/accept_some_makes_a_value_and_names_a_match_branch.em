#$ test: run-pass
#$ rules: GRM-44
#$ stdout: 5
## design rules: R13
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

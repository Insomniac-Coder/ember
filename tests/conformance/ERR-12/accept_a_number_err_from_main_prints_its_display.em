#$ test: run-pass
#$ rules: ERR-12, FN-8
#$ profiles: debug, development
#$ args: x
#$ stderr: error: 2
#$ exit: 1
# D-549 — any `Display` error: a number prints as itself. `main` takes `args`
# and returns a `Result` at once.

fn main(args: Array[String]) -> Result[void, int]:
    return Err(args.len())

#$ test: run-pass
#$ rules: FN-8, FN-10
#$ profiles: debug, development
#$ stdout: fine
# D-549 — a `main` returning `Result[void, E]` that reaches its end returns
# `Ok(())` (`[FN-10]`), and the process exits with status 0.

fn main() -> Result[void, String]:
    println("fine")

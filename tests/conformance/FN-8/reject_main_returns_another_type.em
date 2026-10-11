#$ test: compile-fail
#$ rules: FN-8
#$ help: returns nothing or `Result[void, E]`
# ODR-101 — `main` returns nothing or `Result[void, E]`; a returned number was
# ignored and the process exited with status 0 (D-551).

fn main() -> int: #$ error[E2183]: it returns `i64`
    return 3

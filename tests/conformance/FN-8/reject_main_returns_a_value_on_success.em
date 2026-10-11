#$ test: compile-fail
#$ rules: FN-8
# ODR-101 — a `Result` from `main` has nothing on success: `Result[void, E]`.

fn main() -> Result[int, String]: #$ error[E2183]: it returns `Result[i64, String]`
    return Ok(3)

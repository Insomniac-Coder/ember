#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, release, shipping
#$ panics: reading the answer: `x` is not a number
# `[ERR-4]` — `expect` on an `Err` panics with its message and the error's
# text.

fn parse(text: str) -> Result[int, String]:
    if text == "42":
        return Ok(42)
    return Err(f"`{text}` is not a number")

fn main():
    println(parse("x").expect("reading the answer"))

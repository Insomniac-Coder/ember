#$ test: compile-pass
#$ rules: ERR-5
#$ warning[W2190]: unused `Result`, which may be an error
#$ warning[W2190]: unused `Result`, which may be an error
#$ help: handle it with `?` or `match`, or write `_ = …` to drop it on purpose
# `[ERR-5]` — `Result` is `@must_use`: an expression statement that drops one is `W2190`, a call's
# and a method's alike.

fn parse(s: str) -> Result[int, str]:
    if s == "x":
        return Err("bad")
    return Ok(1)

fn main():
    parse("x")
    parse("1").map(fn(n) => n + 1)

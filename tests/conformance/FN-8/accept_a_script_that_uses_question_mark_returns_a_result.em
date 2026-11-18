#$ test: run-pass
#$ rules: FN-8, ERR-2
#$ stdout: 3
#$ stdout: done
# `[FN-8]` — the implicit `main` is `fn main() -> Result[void, AnyError]` when a statement of the
# script uses `?`: a lambda or a local function in it has its own `?`.

fn parse(text: StrView) -> Result[int]:
    if text == "3":
        return Ok(3)
    return Err(AnyError.msg(owns "not three"))

n = parse("3")?
println(n)
println("done")

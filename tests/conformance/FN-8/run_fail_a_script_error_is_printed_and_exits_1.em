#$ test: run-fail
#$ rules: FN-8, ERR-12
#$ stderr: error: not three
#$ exit: 1
# `[FN-8]` — an `Err` the script's `?` returns from the implicit `main` is printed to standard error
# and the process exits with status 1.

fn parse(text: StrView) -> Result[int]:
    if text == "3":
        return Ok(3)
    return Err(AnyError.msg(owns "not three"))

println("start")
n = parse("4")?
println(n)

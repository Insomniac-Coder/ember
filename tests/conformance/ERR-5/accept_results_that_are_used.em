#$ test: run-pass
#$ rules: ERR-5, ERR-2
#$ stdout: true 1 bad 2
# `[ERR-5]` — a `Result` is used when it is bound, returned, passed, matched, has `?` or a method
# applied; `_ = …` drops one on purpose. None of these warns.

fn parse(s: str) -> Result[int, str]:
    if s == "x":
        return Err("bad")
    return Ok(1)

fn twice(s: str) -> Result[int, str]:
    n = parse(s)?
    return Ok(n * 2)

fn show(r: Result[int, str]) -> str:
    match r:
        Ok(_):
            return "ok"
        Err(e):
            return e

fn main():
    _ = parse("x")
    kept = parse("1")
    ok = parse("1").is_ok()
    match kept:
        Ok(n):
            println(ok, n, show(parse("x")), twice("1").unwrap())
        Err(_):
            println("none")

#$ test: run-pass
#$ rules: ERR-4
#$ profiles: debug, release, shipping
#$ stdout: true false false true
#$ 3 0
#$ true false false true
#$ 42 -1
#$ 42 true `x` is not a number
#$ true 3
# `[ERR-4]` — the tests look without consuming; `ok()`/`err()` keep one side
# as an `Option`; `ok_or` turns an `Option` into a `Result`.

fn parse(text: str) -> Result[int, String]:
    if text == "42":
        return Ok(42)
    return Err(f"`{text}` is not a number")

fn main():
    a: Option[int] = Some(3)
    b: Option[int] = None
    println(a.is_some(), a.is_none(), b.is_some(), b.is_none())
    println(a.unwrap_or_default(), b.unwrap_or_default())
    good = parse("42")
    bad = parse("x")
    println(good.is_ok(), good.is_err(), bad.is_ok(), bad.is_err())
    println(good.unwrap(), bad.unwrap_or(-1))
    println(parse("42").ok().unwrap(), parse("x").ok().is_none(), parse("x").err().unwrap())
    missing: Option[int] = None
    println(missing.ok_or("absent").is_err(), a.ok_or("absent").unwrap())

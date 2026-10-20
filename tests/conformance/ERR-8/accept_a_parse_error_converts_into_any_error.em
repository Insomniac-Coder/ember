#$ test: run-pass
#$ rules: ERR-8, ERR-3, TXT-10
#$ profiles: debug, development
#$ stdout: 7
#$ stdout: the text is not a value of this type
## `[ERR-8]` (D-764) — `?` carries a failed `parse` into an `AnyError`: `ParseError` is an error type
## (`[ERR-3]`), and its text says why the parse failed.

fn parse_one(s: str) -> Result[int]:
    return Ok(s.trim().parse[int]()?)

fn main():
    match parse_one(" 7 "):
        Ok(n) => println(n)
        Err(e) => println(e)
    match parse_one("seven"):
        Ok(n) => println(n)
        Err(e) => println(e)

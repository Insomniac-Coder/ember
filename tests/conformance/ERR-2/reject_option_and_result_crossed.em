#$ test: compile-fail
#$ rules: ERR-2
#$ help: give the `None` an error first: `.ok_or(err)?`
#$ help: let the error go first: `.ok()?`
# `[ERR-2]` — `?` on an `Option` in a function returning a `Result`, or on a
# `Result` in a function returning an `Option`, is `E2181`, whose fix-its are
# `.ok_or(err)?` and `.ok()?`.

fn parse(text: str) -> Result[int, String]:
    if text == "1":
        return Ok(1)
    return Err(String.from("not one"))

fn maybe(n: int) -> Option[int]:
    if n > 0:
        return Some(n)
    return None

fn first(n: int) -> Result[int, String]:
    v = maybe(n)?    #$ error[E2181]: `?` on an `Option[i64]` in a function returning `Result[i64, String]`
    return Ok(v)

fn parsed(text: str) -> Option[int]:
    v = parse(text)?    #$ error[E2181]: `?` on a `Result[i64, String]` in a function returning `Option[i64]`
    return Some(v)

fn main():
    println(first(1).is_ok(), parsed("1").is_some())

#$ test: run-pass
#$ rules: ERR-4, CLO-6
#$ stdout: Some(3)
#$ Some(2)
#$ Some(1)
#$ Some(5)
#$ 7
#$ Ok(1)
#$ Err(2)
#$ Ok(3)
#$ Ok(4)
#$ 6
#$ Ok(8)
# `[ERR-4]` — the methods that take a function take a `once fn` (`[F: needs once fn(owns T) -> U]`):
# each calls it at most once, so it may hand over what it captured. `Option`'s `map`, `and_then`,
# `filter` (which reads), `or_else`, `unwrap_or_else` and `ok_or_else`; `Result`'s `map`, `map_err`,
# `and_then`, `or_else` and `unwrap_or_else`.

fn eat(owns text: String) -> int:
    return text.len()

fn good() -> Result[int, String]:
    return Ok(1)

fn bad() -> Result[int, String]:
    return Err("ab".to_string())

fn ok_of(n: int) -> Result[int, String]:
    return Ok(n)

fn main():
    some: Option[String] = Some("abc".to_string())
    println(some.map(owns owned fn(owns s) => eat(owns s)))
    other: Option[int] = Some(2)
    spare = "xy".to_string()
    println(other.and_then(owns owned fn(owns v) => Some(eat(owns spare)) if v > 1 else None))
    println(other.filter(owns fn(v) => v == 2).map(owns fn(owns v) => v - 1))
    none: Option[int] = None
    fallback = "abcde".to_string()
    println(none.or_else(owns owned fn() => Some(eat(owns fallback))))
    gone = "abcdefg".to_string()
    println(none.unwrap_or_else(owns owned fn() => eat(owns gone)))
    held = "m".to_string()
    rescued: Result[int, int] = none.ok_or_else(owns owned fn() => eat(owns held)).or_else(owns fn(owns e) => Ok(1))
    println(rescued)
    note = "n".to_string()
    println(bad().map_err(owns owned fn(owns e) => eat(owns e)))
    println(good().map(owns owned fn(owns v) => v + eat(owns note) + 1))
    more = "four".to_string()
    println(good().and_then(owns owned fn(owns v) => ok_of(eat(owns more))))
    tail = "abcdef".to_string()
    println(bad().unwrap_or_else(owns owned fn(owns e) => eat(owns tail)))
    last = "abcdefgh".to_string()
    println(bad().or_else(owns owned fn(owns e) => ok_of(eat(owns last))))

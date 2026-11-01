#$ test: run-pass
#$ rules: ERR-4, TYP-23
#$ stdout: Some('ann!')
#$ !
#$ Some(12)
#$ Some(5)
#$ None
#$ 7
#$ Some(9)
#$ Err('missing')
#$ Ok(10)
#$ Err(5)
#$ 5
#$ Err('bad x')
#$ Ok(1)
# ODR-025 — `[ERR-4]`'s methods that take a function. Each consumes its
# receiver and calls the function at most once; the payload is moved in, so
# a lambda needs no mode, and captures stay borrowed (`suffix` is still
# usable after `map`).

fn parse(s: StrView) -> Result[int, String]:
    if s == "1":
        return Ok(1)
    return Err(f"bad {s}")

fn main():
    suffix: String = "!"
    name: Option[String] = Some("ann")
    println(name.map(owns fn(s) => s + suffix))
    println(suffix)
    n: Option[int] = Some(4)
    k = 3
    println(n.map(owns fn(v) => v * k))
    println(n.and_then(owns fn(v) => Some(v + 1) if v > 3 else None))
    println(n.filter(owns fn(v) => v % 2 == 1))
    none: Option[int] = None
    println(none.unwrap_or_else(owns fn() => 7))
    println(none.or_else(owns fn() => Some(9)))
    println(none.ok_or_else(owns fn() => "missing"))
    println(parse("1").map(owns fn(v) => v * 10))
    println(parse("x").map_err(owns fn(e) => e.len()))
    println(parse("x").unwrap_or_else(owns fn(e) => e.len()))
    println(parse("1").and_then(owns fn(v) => parse("x")))
    println(parse("x").or_else(owns fn(e) => parse("1")))

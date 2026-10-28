#$ test: compile-fail
#$ rules: R13
#$ help: to run the block when `r` succeeded write `if Ok(v) = r:`
## R13: a `Result` always holds a value (`Ok` or `Err`), so `exists as` is refused on it (E2036); a
## test of another variant is written as a pattern (`V3/rules.md` section 3's `Result` message).

fn parse(s: StrView) -> Result[int, String]:
    if s == "1":
        return Ok(1)
    return Err(String("not a number"))

fn main():
    r = parse("1")
    if r exists as v:          #$ error[E2036]: `r` is a `Result`, which always holds a value, so this `if` would always run
        println(v)

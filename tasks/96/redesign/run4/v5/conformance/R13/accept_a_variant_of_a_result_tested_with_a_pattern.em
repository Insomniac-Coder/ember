#$ test: run-pass
#$ rules: GRM-44
#$ stdout: 1
## design rules: R13
## R13: the nearest program that is allowed for both E2036 tests: a test of another variant is written
## as a pattern (`if Ok(v) = r:`).

fn parse(s: StrView) -> Result[int, String]:
    if s == "1":
        return Ok(1)
    return Err(String("not a number"))

fn main():
    r = parse("1")
    if Ok(v) = r:
        println(v)

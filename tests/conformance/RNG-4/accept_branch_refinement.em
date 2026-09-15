#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 42

type Percent = i32 in 0 ..= 100
type Upper = i32 in -2147483648 ..= 100

fn as_percent(value: i32) -> Percent:
    if value >= 0 and value <= 100:
        return value
    return Percent.clamped(0)

fn as_upper(value: i32) -> Upper:
    if value > 100:
        return Upper.clamped(100)
    else:
        return value

fn main():
    value = as_percent(42)
    representation: i32 = value
    upper: i32 = as_upper(42)
    println(representation + upper - 42)

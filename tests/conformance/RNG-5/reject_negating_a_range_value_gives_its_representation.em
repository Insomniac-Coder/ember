#$ test: compile-fail
#$ rules: RNG-5, RNG-10, RNG-9
# D-398 — arithmetic on a range value yields its representation (`[RNG-5]`),
# and unary `-` is arithmetic: `-a` of a `Unit` is an `f64`, which is a `Unit`
# again only by a construction (`[RNG-10]`). The negation kept the range type,
# so both lines compiled and held -0.25 and -3, values outside their ranges
# (`[RNG-9]`: invalid, undefined behaviour in safe code).

type Unit = f64 in 0.0 ..= 1.0
type Level = i32 in 0 ..= 10

fn main():
    a = Unit.clamped(0.25)
    l = Level.clamped(3)
    u: Unit = -a        #$ error[E2215]: a `Unit` cannot be built
    m: Level = -l       #$ error[E2215]: a `Level` cannot be built
    x: f64 = u
    y: i32 = m
    println(x, y)

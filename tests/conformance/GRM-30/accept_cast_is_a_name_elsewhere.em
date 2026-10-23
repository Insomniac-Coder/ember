#$ test: run-pass
#$ rules: GRM-30, LEX-15
#$ profiles: debug, development, release
#$ stdout: 3 6
# ODR-280, `[LEX-15]` — `cast` is a keyword only where it starts a downcast, directly before a name:
# elsewhere it is a name, of a function (`cast(2)` is a call) or of a variable.

fn twice(x: int) -> int:
    cast = x * 2
    return cast

fn cast(x: int) -> int:
    return x + 1

fn main():
    println(cast(2), twice(3))

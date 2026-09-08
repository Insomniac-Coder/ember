#$ test: run-pass
#$ rules: STA-1, STA-2

# `[STA-2]` — there are no runtime initialisers, so both of these are values
# the compiler already knows.
const SIZE: usize = 4
const NAME = "ember"
const PI: f32 = 3.14159

static VERSION: i32 = 2

fn main():
    # A `const` may be an array length, which a literal was until now.
    xs: [i32; SIZE] = [7; SIZE]
    println(xs[3])
    total = 0
    for i in 0..SIZE:
        total = total + xs[i]
    println(total)

    println(NAME)
    println(PI)
    println(VERSION)
#$ stdout: 7
#$ 28
#$ ember
#$ 3.14159
#$ 2

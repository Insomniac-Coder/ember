#$ test: run-pass
#$ rules: LEX-16, TYP-5
#$ profiles: debug, release, shipping
#$ stdout: 200
#$ 1.5
#$ 7
# An untyped literal takes the type its context expects, including a float
# type; with no context it is `int`.

fn scale(x: u8) -> u8:
    return x * 2

fn main():
    println(scale(100))
    half: f32 = 1.5
    println(half)
    count: i16 = 7
    println(count)

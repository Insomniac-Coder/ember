#$ test: compile-fail
#$ rules: LEX-16
## `[LEX-16]` - a value that does not fit the resulting type is `E2010`, whatever gave the literal its
## type: an operand, an argument, a collection element, an initialiser, the default `int`.

struct Px:
    r: u8

fn main():
    x: u8 = 200
    z = x + 300                       #$ error[E2010]: the literal `300` does not fit in `u8`
    q = Px(256)                       #$ error[E2010]: the literal `256` does not fit in `u8`
    w: Array[i8] = [128]              #$ error[E2010]: the literal `128` does not fit in `i8`
    u: u8 = -1                        #$ error[E2010]: the literal `-1` does not fit in `u8`
    big = 9223372036854775808         #$ error[E2010]: the literal `9223372036854775808` does not fit in `i64`
    println(z, q.r, w[0], u, big)

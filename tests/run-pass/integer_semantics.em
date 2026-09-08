#$ test: run-pass
#$ rules: TYP-8, TYP-10

@overflow(wrap)
fn wrapped(a: i32, b: i32) -> i32:
    return a + b

@overflow(wrap)
fn shifted(a: u32, n: u32) -> u32:
    return a << n

fn main():
    println(wrapped(2147483647, 1))
    println(shifted(1, 33))
    println(7 / 2)
    println(7 % 2)
    println(-7 / 2)
#$ stdout: -2147483648
#$ 2
#$ 3
#$ 1
#$ -3

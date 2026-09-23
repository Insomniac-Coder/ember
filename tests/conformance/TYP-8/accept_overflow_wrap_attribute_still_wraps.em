#$ test: run-pass
#$ rules: TYP-8
#$ profiles: debug, release, shipping
#$ stdout: -2147483648
# `@overflow(wrap)` asks for wrapping, in every profile.

@overflow(wrap)
fn wrapped(a: i32, b: i32) -> i32:
    return a + b

fn main():
    println(wrapped(2147483647, 1))

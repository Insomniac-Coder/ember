#$ test: run-pass
#$ rules: TYP-7, MOD-5
#$ profiles: debug, release, shipping
#$ stdout: 5

fn null[T](x: i32) -> i32:
    return x + 1

fn main():
    println(null[i32](4))

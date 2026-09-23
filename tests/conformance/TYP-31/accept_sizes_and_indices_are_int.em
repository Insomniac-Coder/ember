#$ test: run-pass
#$ rules: TYP-31
#$ profiles: debug, release, shipping
#$ stdout: 3 5
#$ 30 30 30 10
#$ 2
# `len()` is an `int`, so it meets `int` counters with no cast, and an index
# may be any integer type.

fn main():
    xs = [10, 20, 30]
    s: String = "hello"
    println(xs.len(), s.len())
    last = xs.len() - 1
    small: u8 = 2
    wide: i32 = 2
    println(xs[last], xs[small], xs[wide], xs[0])
    count = 0
    for i in 0..xs.len():
        if xs[i] > 15:
            count += 1
    println(count)

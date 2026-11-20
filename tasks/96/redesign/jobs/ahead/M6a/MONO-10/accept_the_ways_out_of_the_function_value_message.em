# Clause MONO-10.4: "call the function by its name, `add_both(ref mut x, ref mut x)`, or give the
# second name a value of its own: `y = x` above the call and `f(ref mut x, ref mut y)`"
#$ test: run-pass
#$ rules: MONO-10, BRW-11
#$ stdout: 2
#$ stdout: 3 3
# The message's two ways out compile and do what they say.

fn add_both(ref mut a: int, ref mut b: int):
    a += 1
    b += 1

fn main():
    x = 0
    add_both(ref mut x, ref mut x)
    println(x)
    f: fn(ref mut int, ref mut int) = add_both
    y = x
    f(ref mut x, ref mut y)
    println(x, y)

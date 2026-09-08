#$ test: compile-fail
#$ rules: OWN-3
#$ error[E3040]: `xs` has been moved out of

fn main():
    xs: Array[i32] = Array()
    xs.push(1)
    ys = xs
    println(ys.len())
    # `xs` gave its buffer away; reading it again would read a buffer someone
    # else now owns and will free.
    println(xs.len())

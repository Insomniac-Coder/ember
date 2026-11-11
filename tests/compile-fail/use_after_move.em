#$ test: compile-fail
#$ rules: OWN-3
#$ error[E3040]: `xs` was handed over to `ys` on line 8 and is used on line 12; to keep both, write `ys = copy of xs` (a copy of your own) or `ys = shares xs` (two owners of one value)

fn main():
    xs: Array[i32] = Array()
    xs.push(owns 1)
    ys = xs
    println(ys.len())
    # `xs` gave its buffer away; reading it again would read a buffer someone
    # else now owns and will free.
    println(xs.len())

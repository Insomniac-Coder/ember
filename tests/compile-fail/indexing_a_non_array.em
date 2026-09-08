#$ test: compile-fail
#$ rules: IV.3
#$ error[E2020]: cannot index `i32`
#$ error[E2020]: `f32` is not a tuple
#$ error[E2020]: this tuple has 2 elements, so `.2` is out of range
#$ error[E2131]: an array length must be an integer literal
fn main():
    n: i32 = 3
    println(n[0])

    t = (1, 2.0)
    println(t.1.0)
    println(t.2)

    size: usize = 4
    xs: [i32; size] = [0; 4]
    println(xs[0])

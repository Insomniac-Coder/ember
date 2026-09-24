#$ test: compile-fail
#$ rules: SPN-1
#$ error[E3021]: `xs` is borrowed here and mutably borrowed elsewhere
# A slice borrows its source like any view: the `Array` cannot grow while the
# slice is still used.

fn main():
    xs = [1, 2, 3]
    view = xs[0..2]
    xs.push(4)
    println(view)

#$ test: compile-fail
#$ rules: LEX-10, GRM-37
#$ error[E0104]: `#! module overflow(wrap)` is a directive, and directives come first; move it above `fn main()`
## `[LEX-10]` - a continuation line inside brackets that begins with `#!` is a directive as well.

fn main():
    x = [1,
        #! module overflow(wrap)
        2]
    println(x.len())

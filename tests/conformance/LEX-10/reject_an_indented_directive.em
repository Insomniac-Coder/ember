#$ test: compile-fail
#$ rules: LEX-10, GRM-37
#$ error[E0104]: `#! module overflow(wrap)` is a directive, and directives come first; move it above `fn main()`
## `[LEX-10]` - a line whose first characters after its indentation are `#!` is a directive, never a
## comment: inside a function body it is `E0104` too.

fn main():
    #! module overflow(wrap)
    println(1)

#$ test: compile-fail
#$ rules: LEX-10, GRM-37
#$ error[E0104]: `#! module overflow(wrap)` is a directive, and directives come first; move it above `import std.io`
## `[LEX-10]` - a line beginning `#!` is a directive wherever it stands and never a comment: one below an
## import is `E0104`, and the message names the line to move it above.

import std.io

#! module overflow(wrap)

fn main():
    println(1)

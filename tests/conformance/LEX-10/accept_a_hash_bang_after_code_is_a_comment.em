#! language "0.9.20"
#$ test: run-pass
#$ rules: LEX-10, GRM-37
#$ stdout: 1
## `[LEX-10]` - a line that BEGINS with `#!` is a directive; `#!` after code on a line is an ordinary
## comment. There are no block comments.

fn main():
    println(1)    #! also a comment

#! language "0.9.10"
#$ test: run-pass
#$ rules: LEX-10, GRM-37
#$ stdout: 1
## `[LEX-10]` — `#!` before the first item is a directive; anywhere else it
## begins a comment. There are no block comments.

fn main():
    #! a comment, not a directive
    println(1)    #! also a comment

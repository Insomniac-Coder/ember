#$ test: run-pass
#$ rules: LEX-5, LEX-8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
## `[LEX-5]` — a deeper line opens a level and a shallower one closes every
## level above it; `[LEX-8]` — blank and comment-only lines, at any column,
## take no part.

fn main():
    if true:
        if true:
            println(1)

# a comment at column 0
                # and one deeper than anything
        println(2)
    println(3)

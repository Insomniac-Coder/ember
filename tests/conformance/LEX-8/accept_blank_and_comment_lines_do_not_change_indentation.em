#$ test: run-pass
#$ rules: LEX-8
#$ stdout: 3
# `[LEX-8]` — blank lines and comment-only lines do not affect indentation: the comment below is at
# column 0 and one deeper than the body, and the block goes on.

fn main():
    x = 1

# a comment at column 0 inside the body
        # a comment deeper than the body
    x = x + 2

    println(x)

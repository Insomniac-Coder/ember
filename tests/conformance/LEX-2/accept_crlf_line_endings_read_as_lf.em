#$ test: run-pass
#$ rules: LEX-2
#$ stdout: 6
# `[LEX-2]` -- line endings are LF or CRLF, both normalised to LF before tokenisation.

fn main():
    x = 1 + 2
    println(x * 2)

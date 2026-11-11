#$ test: compile-fail
#$ rules: LEX-1
#$ error[E0001]: is not valid UTF-8
# `[LEX-1]` - a source file is UTF-8: one that holds a byte that is not is E0001, naming the file.

from support.not_utf8 import one

fn main():
    println(one())

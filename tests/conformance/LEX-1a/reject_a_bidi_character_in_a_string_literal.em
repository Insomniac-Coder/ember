#$ test: compile-fail
#$ rules: LEX-1a
## `[LEX-1a]` - in a string literal: E0012, whose help is the escape `\u{2066}`.

fn main():
    println("isolate ⁦ inside")    #$ error[E0012]: line 6 holds U+2066
#$ help: write the text as an escape

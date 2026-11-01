#$ test: compile-fail
#$ rules: LEX-14
## `[LEX-14]` — a keyword where the grammar expects that keyword is no name: in `for in xs:` the
## pattern is left out, and the error is the one for that, `E0100 expected a pattern`, with nothing
## after it; `E0005` is for a keyword written as a name (`for type in xs:`).

fn main():
    xs = [1, 2]
    for in xs:    #$ error[E0100]: expected a pattern
        println(1)
    for type in xs:    #$ error[E0005]: `type` is a keyword
        println(type)    #$ error[E0005]: `type` is a keyword

#$ test: compile-fail
#$ rules: R15, R16
## R15: "`Send` is declared only by an `unsafe extend` author" (today's message, kept).

struct Raw:
    p: *mut int

extend Raw implements Send:    #$ error[E3100]: implementing `Send` by hand requires `unsafe extend`: the compiler cannot check it
    pass

fn main():
    println(1)

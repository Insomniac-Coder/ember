#$ test: compile-fail
#$ rules: GRM-19
## `[GRM-19]` — the pattern of a condition MUST be refutable: an irrefutable one is `E2036`, and for a
## bare name already in scope the primary help is "to compare, write `if x == 5:`".

fn main():
    x = 3
    if x = 5:    #$ error[E2036]: this pattern always matches
                 #$ help: to compare, write `if x == 5:`
        println(1)
    while x = 4:    #$ error[E2036]: this pattern always matches
        println(2)
    if (c, d) = (3, 4):    #$ error[E2036]: this pattern always matches
        println(c, d)

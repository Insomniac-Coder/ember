#$ test: compile-fail
#$ rules: GRM-36
## `[GRM-36]` — `ref` of any other expression is `E0111`, whose help is to bind the value to a local first.

fn main():
    xs = [1, 2]
    e = ref 5    #$ error[E0111]: only a place can be borrowed
                 #$ help: bind the value to a local first
    f = ref (1 + 2)    #$ error[E0111]: only a place can be borrowed
    g = ref xs.len()    #$ error[E0111]: only a place can be borrowed
    println(e, f, g)

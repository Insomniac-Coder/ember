#$ test: compile-fail
#$ rules: GRM-5, GRM-4
## `[GRM-5]` — a name that appears twice in one target list is `E1020`.

fn main():
    a, a = (1, 2)    #$ error[E1020]: `a` is declared more than once in this destructuring
    println(a)

#$ test: compile-fail
#$ rules: GRM-41, GRM-13
## `[GRM-41]` — `mut` alone on a lambda parameter or on a pattern name is `E0113`,
## whose message gives the line with `ref mut` in the program's own names.

fn main():
    f = fn(mut x: int): x += 1    #$ error[E0113]: `mut` alone is no word
    o: Option[int] = Some(1)
    match o:
        Some(mut v):    #$ error[E0113]: `mut` alone is no word
            println(v)
        None:
            pass
    k = 1
    f(ref mut k)
    println(k)

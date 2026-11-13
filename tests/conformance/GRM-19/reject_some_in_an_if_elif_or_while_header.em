#$ test: compile-fail
#$ rules: GRM-19, GRM-44
## `[GRM-19]` — `Some(...)` as the pattern of an `if`, `elif` or `while` header is `E0115`, since an
## `Option` is opened with `exists as` (`[GRM-44]`).

fn main():
    o: Option[int] = Some(1)
    if Some(a) = o:    #$ error[E0115]: an `Option` is opened with `exists as`
        println(a)
    elif Some(b) = o:    #$ error[E0115]: an `Option` is opened with `exists as`
        println(b)
    while Some(c) = o:    #$ error[E0115]: an `Option` is opened with `exists as`
        println(c)
        o = None

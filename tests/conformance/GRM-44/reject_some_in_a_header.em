#$ test: compile-fail
#$ rules: GRM-44, GRM-19
## `[GRM-19]`, `[GRM-44]` — an `Option` is opened with `exists as`; `Some(...)` in an `if` or `while`
## header is `E0115`, its help the line with `exists as`.

fn main():
    h: Option[int] = Some(1)
    if Some(v) = h:    #$ error[E0115]: an `Option` is opened with `exists as`, not `Some(...)` in a header
        println(v)

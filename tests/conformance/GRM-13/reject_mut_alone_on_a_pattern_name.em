#$ test: compile-fail
#$ rules: GRM-13
## `[GRM-13]` — a name a pattern declares takes `ref mut` to change the part, or `copy of` for a copy
## of its own; `mut` alone is `E0113`.

fn main():
    h: Option[int] = Some(1)
    match h:
        Some(mut v):    #$ error[E0113]: `mut` alone is no word for a name a pattern declares
            v += 1
        None:
            pass

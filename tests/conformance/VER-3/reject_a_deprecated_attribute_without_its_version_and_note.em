#$ test: compile-fail
#$ rules: VER-3, ATT-1
## `[VER-3]` — `@deprecated(since, note)` is the one form (F-020): the version the item was
## deprecated in and a note, both text, by position or by name. Anything else is `E0104`.

@deprecated    #$ error[E0104]: `@deprecated` takes the version it was deprecated in and a note
fn a() -> int:
    return 1

@deprecated(since="1.0")    #$ error[E0104]: `@deprecated` takes the version it was deprecated in and a note
fn b() -> int:
    return 2

@deprecated(since=1, note="use `a`")    #$ error[E0104]: `@deprecated` takes the version
fn c() -> int:
    return 3

@deprecated(version="1.0", note="use `a`")    #$ error[E0104]: `@deprecated` takes the version
fn d() -> int:
    return 4

fn main():
    println(a() + b() + c() + d())

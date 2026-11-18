#$ test: compile-fail
#$ rules: LEX-20, TXT-9
## `[LEX-20]` - only the literal produces a `String` where one is expected; a `StrView` value (a name
## bound to a literal) is a view and never converts implicitly.

fn own(s: String) -> int:
    return s.len()

fn main():
    a = "text"
    println(own(a))     #$ error[E2020]: expected `String`, found `StrView`
    c: String = a       #$ error[E2020]: expected `String`, found `StrView`
    println(c)

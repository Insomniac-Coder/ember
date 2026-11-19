#$ test: compile-fail
#$ rules: ERR-3, ERR-8
# `[ERR-3]` — `@from` on a variant with no field, two fields, or a field of another type (`AnyError`
# included, `[ERR-8]`) is `E2080`, naming the field.

@derive(Error)
enum Empty:
    @error("none")
    @from    #$ error[E2080]: `@from` marks a variant with one field, and `Empty.Nothing` has no field
    Nothing

@derive(Error)
enum Wraps:
    @error("any")
    @from
    Any(AnyError)    #$ error[E2080]: payload `_0` has type `AnyError`, which is not Error

@derive(Error)
enum Named:
    @error("named {code}")
    @from
    Coded(code: String)    #$ error[E2080]: payload `code` has type `String`, which is not Error

fn main():
    println(1)

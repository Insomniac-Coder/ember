#$ test: compile-fail
#$ rules: ERR-3, ATT-1
# `[ERR-3]` — `@derive(Error)` needs an `@error("…")` text on each variant (on
# the struct, for a struct), whose placeholders name fields; `@from` marks a
# variant with one field, an error type (`E2080` otherwise, naming the field); neither attribute means anything
# elsewhere.

@derive(Error)
enum Failure:
    @error("lost")
    Lost
    Gone    #$ error[E0104]: `@derive(Error)` needs the text of `Failure.Gone`

@derive(Error)
enum Bad:
    @error("at {line}")    #$ error[E1010]: the `@error` text names `{line}`, which is none of the fields
    At(row: int)
    @error("pair")
    @from    #$ error[E2080]: `@from` marks a variant with one field, and `Bad.Pair` has 2 fields
    Pair(a: int, b: int)

@derive(Error)
struct Silent:    #$ error[E0104]: `@derive(Error)` needs the text of `Silent`
    code: int

@derive(Error)
enum Wrapped:
    @error("wrapped")
    @from
    Number(int)    #$ error[E2080]: payload `_0` has type `i64`, which is not Error

enum Plain:
    @error("plain")    #$ error[E0104]: `@error` applies to a variant of an enum deriving `Error`
    One

fn main():
    println(1)

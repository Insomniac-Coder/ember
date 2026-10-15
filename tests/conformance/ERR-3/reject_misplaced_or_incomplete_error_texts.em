#$ test: compile-fail
#$ rules: ERR-3, ATT-1
# `[ERR-3]` — `@derive(Error)` needs an `@error("…")` text on each variant (on
# the struct, for a struct), whose placeholders name fields; `@from` marks a
# variant with one field, an error type; neither attribute means anything
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
    @from    #$ error[E0104]: `@from` marks a variant with one field
    Pair(a: int, b: int)

@derive(Error)
struct Silent:    #$ error[E0104]: `@derive(Error)` needs the text of `Silent`
    code: int

@derive(Error)
enum Wrapped:
    @error("wrapped")
    @from    #$ error[E2020]: expected `ref dyn std.core.Error`, found `ref i64`
    Number(int)

enum Plain:
    @error("plain")    #$ error[E0104]: `@error` applies to a variant of an enum deriving `Error`
    One

fn main():
    println(1)

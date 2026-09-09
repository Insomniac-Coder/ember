#$ test: parse-fail
#$ rules: LEX-15b
## `yield` is fully reserved in v1: a keyword everywhere, with `r#yield`
## required to use the word as a name. It is not `E0005` — that code names the
## version a word will arrive in, and `yield` arrives in this one.

fn f():
    yield = 1          #$ error[E0100]: unexpected token

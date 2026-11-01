#$ test: parse-fail
#$ rules: LEX-15b, LEX-14
## `yield` is a keyword of the table, with `r#yield` required to use the word as a name: written
## as a name it is `E0005`, "`yield` is a keyword", as every keyword of the table is (`[LEX-14]`).

fn f():
    yield = 1          #$ error[E0005]: `yield` is a keyword

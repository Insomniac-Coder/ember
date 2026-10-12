#$ test: parse-fail
#$ rules: LEX-9
## `[LEX-9]` — a `:` at the end of a line opens an indented block; a line that
## does not indent is `E0004`.

fn main():
    if true:    #$ error[E0004]: expected an indented block
    println(1)

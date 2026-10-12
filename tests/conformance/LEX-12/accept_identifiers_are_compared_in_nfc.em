#$ test: run-pass
#$ rules: LEX-12
#$ stdout: 1
## `[LEX-12]` — identifiers are NFC-normalised: `café` with a precomposed
## `é` and `café` with a combining accent are one name.

fn main():
    café = 1
    println(café)

#$ test: run-pass
#$ rules: LEX-18
#$ stdout: 7 1.5
## `[LEX-18]` — `5.` followed by an identifier is a method call on `5`; `1.5`
## is a float.

fn main():
    println(3.add(4), 1.5)

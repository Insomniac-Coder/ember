#$ test: run-pass
#$ rules: GRM-12, CT-1
#$ stdout: limit low untyped computed other
#$ stdout: done
#$ stdout: not star
## `[GRM-12]` — an identifier in pattern position names a `const` when one of
## that name is in scope, and matches its value; it bound a new name instead,
## which matched everything (D-577).

const LIMIT: int = 3
const LOW: int = -2
const UNTYPED = 7
const DONE: bool = true
const STAR: char = '*'
const COMPUTED: int = LIMIT * 2

fn kind(x: int) -> str:
    match x:
        LIMIT:
            return "limit"
        LOW:
            return "low"
        UNTYPED:
            return "untyped"
        COMPUTED:
            return "computed"
        _:
            return "other"

fn main():
    println(kind(3), kind(-2), kind(7), kind(6), kind(4))
    match true:
        DONE:
            println("done")
        _:
            println("not done")
    match 'x':
        STAR:
            println("star")
        _:
            println("not star")

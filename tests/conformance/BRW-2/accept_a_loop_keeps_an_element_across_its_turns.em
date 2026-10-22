#$ test: run-pass
#$ rules: BRW-2, BRW-1
#$ stdout: bb
# The "best so far" loop: `best` borrows an element of `xs` and lives on through the next turns.
# Each turn binds `x` afresh (a reference local is never re-seated, `[BRW-1]`), which writes no
# element, so nothing conflicts with the borrow `best` keeps; it is live to the `println`.

fn main():
    xs: Array[String] = ["a", "bb", "c"]
    best: str = ""
    for x in xs:
        if x.len() > best.len():
            best = x
    println(best)

#$ test: run-fail
#$ rules: ERR-13, ERR-4
#$ panics: called unwrap on None
# `[ERR-13]` — `unwrap` on `None` is the caller's bug: it panics.

fn main():
    o: Option[int] = None
    println(o.unwrap())

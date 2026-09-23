#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, release, shipping
#$ panics: no such file
# `[ERR-4]` — `unwrap` on an `Err` panics with the error's text.

fn main():
    opened: Result[int, str] = Err("no such file")
    println(opened.unwrap())

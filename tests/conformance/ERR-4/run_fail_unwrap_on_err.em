#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, development, release
#$ panics: no such file
# `[ERR-4]` — `unwrap` on an `Err` panics with the error's text.

fn main():
    opened: Result[int, StrView] = Err("no such file")
    println(opened.unwrap())

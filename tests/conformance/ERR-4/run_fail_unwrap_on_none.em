#$ test: run-fail
#$ rules: ERR-4
#$ profiles: debug, development, release
#$ panics: called unwrap on None
# `[ERR-4]` — on an `Option`, `unwrap` panics with `called unwrap on None`.

fn main():
    port: Option[int] = None
    println(port.unwrap())

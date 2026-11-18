#$ test: run-fail
#$ rules: ERR-13
#$ panics: key not found: 3; use .get(k) for an Option
# `[ERR-13]` — `m[k]` for a missing key panics, and its message names `m.get(k)`.

fn main():
    m: Map[int, int] = {}
    m.insert(owns 1, owns 1)
    println(m[3])

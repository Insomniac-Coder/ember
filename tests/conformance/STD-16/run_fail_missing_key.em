#$ test: run-fail
#$ rules: STD-16, STD-17
#$ panics: key not found: 'zz'; use .get(k) for an Option
# `[STD-16]` — `m[k]` on a missing key panics, naming the key by its `Debug`.

fn main():
    m: Map[String, int] = {"a": 1}
    println(m["zz"])

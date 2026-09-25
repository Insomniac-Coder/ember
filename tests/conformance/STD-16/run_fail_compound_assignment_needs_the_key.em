#$ test: run-fail
#$ rules: STD-16, STD-17
#$ panics: key not found: 'b'; use .get(k) for an Option
# `[STD-16]`, `[STD-17]` — `m[k] += 1` goes through `index_mut`: the key must
# be there, and a missing one panics as `m[k]` does.

fn main():
    m: Map[String, int] = {"a": 1}
    m["a"] += 1
    m["b"] += 1
    println(m)

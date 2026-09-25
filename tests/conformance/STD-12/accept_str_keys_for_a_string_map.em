#$ test: run-pass
#$ rules: STD-12, STD-16, STD-17
#$ stdout: Some(1) true 1
#$ stdout: None
#$ stdout: {'a': 2, 'b': 3}
# `[STD-12]` (ODR-032) — a `Map[String, V]` is looked up with a `str`
# (`AsKey[String]`, whose `is_key` compares without converting), and
# `m[s] = v` converts the `str` with `to_key` only when the key is new.

fn main():
    m: Map[String, int] = {"a": 1}
    key: str = "a"
    found = m.get(key)
    println(found, key in m, m[key])
    println(m.remove("zz"))
    m[key] = 2
    m["b"] = 3
    println(m)

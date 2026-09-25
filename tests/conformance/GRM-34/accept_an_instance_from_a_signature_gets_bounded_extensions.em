#$ test: run-pass
#$ rules: GRM-34, STD-16
#$ stdout: 3 a b
# `[GRM-34]` — a generic type's instance made by a function signature, before
# the implementations its extensions' bounds need were collected, still gets
# those extensions: `Map[String, int]`'s `keys` needs `String: Hash`.

fn total(m: Map[String, int]) -> int:
    n = 0
    for k in m.keys():
        n += m[k]
    return n

fn names(m: Map[String, int]) -> String:
    out = String.from("")
    for k in m:
        out += f" {k}"
    return out

fn main():
    listed = names({"a": 1, "b": 2})
    println(total({"a": 1, "b": 2}), listed.trim())

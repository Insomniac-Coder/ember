#$ test: run-pass
#$ rules: GRM-13, STD-17
#$ stdout: {'a': Some('hi!'), 'b': None}
#$ Some('hi!')
# D-295 — `match m[k]: Some(ref mut s)` writes the map's value in place:
# the scrutinee goes through `index_mut`, as `ref mut m[k]` does.

fn main():
    m: Map[String, Option[String]] = {"a": Some("hi"), "b": None}
    match m["a"]:
        Some(ref mut s):
            s += "!"
        None:
            pass
    println(m)
    x: Option[String] = Some("hi")
    match x:
        Some(ref mut s):
            s += "!"
        None:
            pass
    println(x)

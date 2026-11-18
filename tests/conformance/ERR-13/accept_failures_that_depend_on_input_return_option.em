#$ test: run-pass
#$ rules: ERR-13
#$ stdout: None
#$ stdout: None
#$ stdout: Some(2)
# `[ERR-13]` — an operation whose failure depends on the world or on input returns `Option` or
# `Result`: `xs.get(i)`, `m.get(k)`, `parse`.

fn main():
    xs = [1, 2, 3]
    m: Map[int, int] = {}
    println(xs.get(7))
    println(m.get(3))
    println(xs.get(1))

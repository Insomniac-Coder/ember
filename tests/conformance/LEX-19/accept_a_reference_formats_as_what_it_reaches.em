#$ test: run-pass
#$ rules: LEX-19, SPN-5
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("const int64_t*") == 0
#$ stdout:
#$ 1 (0, 1)
#$ 2 (1, 2)
#$ a|'a'|(0, 'a')
#$ 20
# `[LEX-19]`, D-385 — an f-string writes a reference as what it reaches,
# wherever it comes from: a tuple's `ref` part (`p.1` of an `(int, ref int)`)
# was given to C as text. D-386 — a view iterator's `next` gives a `ref T`,
# which is `T*` in C for a shared view too; it gave `const T*` (C4090).

fn main():
    ys: Array[int] = [1, 2]
    names: Array[String] = ["a"]
    it = ys.iter().enumerate()
    for p in it:
        println(f"{p.1} {p}")
    for q in names.iter().enumerate():
        println(f"{q.1}|{q.1!r}|{q}")
    xs: Array[int] = [10, 20]
    walk = xs.iter()
    walk.next()
    match walk.next():
        Some(x):
            println(f"{x}")
        None:
            println("none")

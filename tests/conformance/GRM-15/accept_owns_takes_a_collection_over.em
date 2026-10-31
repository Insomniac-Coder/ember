#$ test: run-pass
#$ rules: GRM-15, GRM-27, CTL-1
#$ stdout: ab
#$ stdout: 2
#$ stdout: c
## `[GRM-15]` — `owns e` before a `for` loop's iterable, a comprehension's and a `match`'s
## scrutinee takes the value over: the loop or the branches have it, and the name ends there.

enum Shape:
    Named(String)
    Empty

fn main():
    words: Array[String] = [String("a"), String("b")]
    joined = String()
    for w in owns words:
        joined.push_str(w)
    println(joined)
    more: Array[String] = [String("x"), String("yy")]
    sizes = [w.len() for w in owns more]
    println(sizes[1])
    s = Shape.Named(String("c"))
    match owns s:
        Shape.Named(name):
            println(name)
        Shape.Empty:
            pass

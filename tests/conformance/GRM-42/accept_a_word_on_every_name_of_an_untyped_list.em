#$ test: run-pass
#$ rules: GRM-42, GRM-13
#$ stdout: 1 2
#$ stdout: 6
## `[GRM-42]` — each word belongs to the one name after it: a word on every name of a loop's names, of a
## pattern's names and of a lambda's untyped parameters, and `ref` (a name's default) on any declared name.

fn main():
    pairs = [(1, 2)]
    for ref a, ref b in pairs:
        println(a, b)
    add = fn(ref mut x: int, ref y: int): x += y
    n = 4
    add(ref mut n, 2)
    println(n)

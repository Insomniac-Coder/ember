#$ test: run-pass
#$ rules: TYP-14
#$ stdout: 3
## `[TYP-14]` - a child held in its own block (`owns`, a list) is not a type with no end.

struct Fine:
    left: Option[owns Fine]
    more: Array[Fine]
    n: int

fn count(f: Fine) -> int:
    t = 1
    if f.left exists as l:
        t += count(l)
    for m in f.more:
        t += count(m)
    return t

fn main():
    f = Fine(left=owns Fine(left=None, more=[], n=1), more=[Fine(left=None, more=[], n=2)], n=0)
    println(count(f))

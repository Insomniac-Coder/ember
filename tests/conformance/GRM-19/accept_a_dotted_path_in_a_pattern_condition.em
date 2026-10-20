#$ test: run-pass
#$ rules: GRM-19
#$ stdout: fast
#$ stdout: slow 3
#$ stdout: shape 2
## `[GRM-19]` (D-776) — a pattern condition takes every pattern a `match` arm takes: a variant
## named by a dotted path, alone or inside a constructor, and a constructor named by one.

enum Mode:
    Slow
    Fast

enum Shape:
    Dot
    Square(int)

fn main():
    kept: Option[Mode] = Some(Mode.Fast)
    if Some(Mode.Fast) = kept:
        println("fast")
    m = Mode.Slow
    count = 0
    while Mode.Slow = m:
        count += 1
        if count == 3:
            m = Mode.Fast
    if Mode.Fast = m:
        println("slow", count)
    if Shape.Square(side) = Shape.Square(2):
        println("shape", side)

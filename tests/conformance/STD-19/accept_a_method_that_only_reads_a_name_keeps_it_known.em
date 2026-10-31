#$ test: run-pass
#$ rules: STD-19, CLS-7
#$ stdout:
#$ 10
#$ 20
# `[STD-19]`, `[CLS-7]` — what a method does to its object is worked out from its body, and the
# known-numbers rule follows a name only when nothing changes it. `show`, written after its call, only
# reads the range it is called on, so `kept` still holds `-10 .. 10`: it has 20 items, an `int`.

interface Show:
    fn show()

fn main():
    kept = -10 .. 10
    kept.show()
    e: int = kept.iter().len()
    println(e)

extend Range[int] implements Show:
    fn show():
        println(this.end)

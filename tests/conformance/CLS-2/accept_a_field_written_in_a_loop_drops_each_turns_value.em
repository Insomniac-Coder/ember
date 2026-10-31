#$ test: run-pass
#$ rules: CLS-2, OWN-5
#$ profiles: debug, development
#$ stdout: dropped 0
#$ stdout: dropped 1
#$ stdout: last 2
#$ stdout: dropped 2
# `[CLS-2]` — a field a constructor writes again in a loop holds a value on every turn, so each
# turn's write drops the one before; the last stays, and goes with the object.

class Tag:
    n: int

    fn drop():
        println(f"dropped {this.n}")

class Holder:
    tag: Tag

    fn init(turns: int):
        this.tag = Tag(n=0)
        for i in range(1, turns):
            this.tag = Tag(n=i)

fn main():
    h = Holder(3)
    println("last", h.tag.n)

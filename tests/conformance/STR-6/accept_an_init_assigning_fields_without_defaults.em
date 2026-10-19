#$ test: run-pass
#$ rules: STR-6, CLS-2
#$ profiles: debug, development
#$ stdout: P(x=1, label='one', tags=['a'], scale=2)
#$ stdout: P(x=-4, label='negative', tags=[], scale=2)
#$ stdout: dropped first
#$ stdout: second
#$ stdout: dropped second
# `[STR-6]` — a struct's `init` is its constructor: a field without a default is assigned on every
# path before `self` is used as a whole, and a write to it again replaces the value it held, which
# is dropped first (`[CLS-2]`).

class Tag:
    name: String

    fn drop(mut self):
        println(f"dropped {self.name}")

struct P:
    x: int
    label: String
    tags: Array[String]
    scale: int = 2

    fn init(mut self, x: int):
        self.x = x
        if x < 0:
            self.label = "negative"
            self.tags = []
        else:
            self.label = "one"
            self.tags = ["a"]

struct Holder:
    tag: Tag

    fn init(mut self):
        self.tag = Tag(name="first")
        self.tag = Tag(name="second")

fn main():
    println(f"{P(1)!r}")
    println(f"{P(-4)!r}")
    h = Holder()
    println(h.tag.name)

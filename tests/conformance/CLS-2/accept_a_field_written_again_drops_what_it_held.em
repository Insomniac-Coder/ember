#$ test: run-pass
#$ rules: CLS-2, OWN-5, CLS-6
#$ profiles: debug, development
#$ stdout: dropped first
#$ stdout: dropped draft
#$ stdout: second final 42
#$ stdout: dropped final
#$ stdout: dropped second
#$ stdout: dropped first
#$ stdout: second final 7
#$ stdout: dropped final
#$ stdout: dropped second
# `[CLS-2]` — a constructor may write a field twice, or after a write on only some paths: the
# later write replaces a value, which is dropped first (`[OWN-5]`: evaluate, drop, store), and
# drops nothing where no earlier write ran. Each `Tag` is dropped exactly once; an object's fields
# in reverse order (`[CLS-6]`).

class Tag:
    name: String

    fn drop():
        println(f"dropped {this.name}")

class Pair:
    left: Tag
    right: Tag
    count: i32

    fn init(rewrite: bool, count: i32):
        this.left = Tag(name="first")
        this.left = Tag(name="second")
        if rewrite:
            this.right = Tag(name="draft")
        this.right = Tag(name="final")
        this.count = count

fn report(p: Pair):
    println(p.left.name, p.right.name, p.count)

fn main():
    report(Pair(true, 42))
    report(Pair(false, 7))

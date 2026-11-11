#$ test: run-pass
#$ rules: CTL-6, DRP-1
#$ stdout: body a b
#$ stdout: drop b
#$ stdout: drop a
#$ stdout: after
# `[CTL-6]` — `with a = e1, b = e2:` binds `a` and `b` for the block and drops them in reverse order
# when it ends.

struct Guard:
    name: String

    fn drop():
        println("drop", this.name)

fn main():
    with a = Guard(name=String("a")), b = Guard(name=String("b")):
        println("body", a.name, b.name)
    println("after")

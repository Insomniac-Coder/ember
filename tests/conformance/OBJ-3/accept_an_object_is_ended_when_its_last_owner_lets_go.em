#$ test: run-pass
#$ rules: OBJ-3, CLS-6
#$ stdout: made
#$ stdout: in scope
#$ stdout: ended
#$ stdout: after
# `[OBJ-3]` — when the count of owners reaches 0 the object is ended: its `drop` chain and its field
# drops run.

class Token:
    name: String

    fn drop():
        println("ended")

fn scoped():
    t = Token(name=String("t"))
    println("made")
    println("in scope")

fn main():
    scoped()
    println("after")

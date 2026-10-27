#$ test: run-pass
#$ rules: OWN-1
#$ stdout: handed over
#$ end one
## design rules: R1
## R1: a hand-over moves the one owner; the object is ended exactly once, by the name that owns it
## when it ends (`b`), never by the name it was handed over from (`a`).

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    a = Res(name=String("one"))
    b = a
    println("handed over")

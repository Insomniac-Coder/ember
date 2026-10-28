#$ test: run-pass
#$ rules: R5
#$ stdout: made
#$ stdout: end a
## R5, verdict 69: the nearest allowed program: `fn drop():` reads its object and changes nothing
## outside it; `r` ends with `main`, after the print.

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    r = Res(name=String("a"))
    println("made")

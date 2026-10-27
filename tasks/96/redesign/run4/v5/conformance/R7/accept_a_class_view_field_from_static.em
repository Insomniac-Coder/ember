#$ test: run-pass
#$ rules: LT-45
#$ stdout: ann human
## design rules: R7
## R7: a view field of a class may say `from static`: it holds only text written in the program, so it
## lives as long as any object. AN-26's NEW program.

class Player:
    name: String
    kind: StrView from static

    fn init(name: StrView):
        this.name = String(name)
        this.kind = "human"

fn make(s: String) -> Player:
    return Player(s)

fn main():
    p = make(String("ann"))
    println(p.name, p.kind)

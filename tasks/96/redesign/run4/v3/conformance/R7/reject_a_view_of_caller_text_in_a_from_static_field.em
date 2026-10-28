#$ test: compile-fail
#$ rules: R7
#$ help: this.kind = String(name)
## R7: a `from static` field takes only text written in the program; a view of the caller's text is
## refused (E3063). AN-26, point 6.

class Player:
    name: String
    kind: StrView from static

    fn init(name: StrView):
        this.name = String(name)
        this.kind = name    #$ error[E3063]: `kind` holds only text written in the program, and `name` comes from the caller; write `this.kind = String(name)` with `kind: String`

fn main():
    p = Player("ann")
    println(p.kind)

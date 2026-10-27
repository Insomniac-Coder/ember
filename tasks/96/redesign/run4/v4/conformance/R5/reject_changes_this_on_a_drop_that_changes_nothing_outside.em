#$ test: compile-fail
#$ rules: R5
#$ help: fn drop():
## R5, verdict 69: a `drop` that only reads or cleans up its object says neither `changes this` nor
## anything else (E2122).

class Res:
    name: String

    fn drop() changes this:    #$ error[E2122]: remove `changes this`: a `drop` always changes its own object, and this one changes nothing outside it; write `fn drop():`
        println("end", this.name)

fn main():
    r = Res(name=String("a"))
    println("made")

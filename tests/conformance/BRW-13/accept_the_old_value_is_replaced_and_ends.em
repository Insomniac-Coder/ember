# Clause BRW-13.1: "`=` on a name that has a value puts the new value in the place the name stands
# for. A name declared with `=` stands for its own place: its old value is replaced (and ends, if it
# owned something)"
#$ test: run-pass
#$ rules: BRW-13, OWN-5, OWN-2
#$ stdout: end first
#$ stdout: after second
#$ stdout: end second

struct Noisy:
    name: String
    fn drop():
        println("end", this.name)

fn main():
    t = Noisy(name=String("first"))
    t = Noisy(name=String("second"))
    println("after", t.name)

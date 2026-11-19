#$ test: run-pass
#$ rules: EXP-4, OWN-3
#$ profiles: debug, development, release
#$ stdout: end 1
#$ taken 1
#$ done
# `[EXP-4]`, `[OWN-3]` — a temporary the right side of an `and` makes exists only on the turns that reach
# it, and ends once on those: a later turn that does not make it ends nothing.

struct Loud:
    n: int

    fn drop():
        println("end", this.n)

fn main():
    for i in 0..4:
        if i == 1 and Loud(i).n > 0:
            println("taken", i)
    println("done")

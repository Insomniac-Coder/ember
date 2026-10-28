#$ test: run-pass
#$ rules: BRW-12
#$ stdout: end one
#$ middle
#$ end two
## design rules: R2
## R2: `=` on a name that already has a value puts the new value in the place the name stands for:
## its old value is replaced and ends there, once (object destruction).

class Res:
    name: String

    fn drop():
        println("end", this.name)

fn main():
    a = Res(name=String("one"))
    a = Res(name=String("two"))
    println("middle")

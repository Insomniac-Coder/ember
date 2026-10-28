#$ test: run-pass
#$ rules: R5
#$ stdout: a
#$ stdout: 1 a
## R5, verdicts 69, 57: the nearest allowed program: `fn drop() changes LOG:` names only the change
## outside its object; `use_one`, whose local `r` ends with it and so calls `drop`, says `changes LOG`,
## and so does `main`, which calls `use_one`.

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop() changes LOG:
        LOG.push(copy of this.name)

fn use_one() changes LOG:
    r = Res(name=String("a"))
    println(r.name)

fn main() changes LOG:
    use_one()
    println(LOG.len(), LOG[0])

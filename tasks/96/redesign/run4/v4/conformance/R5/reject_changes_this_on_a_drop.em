#$ test: compile-fail
#$ rules: R5
#$ help: fn drop() changes LOG:
## R5, verdict 69: changing its own object is a given for `drop`, so its first line names only what it
## changes outside it; `changes this` there is refused (E2122).

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop() changes this, LOG:    #$ error[E2122]: remove `changes this`: a `drop` always changes its own object; name only what it changes outside it: `fn drop() changes LOG:`
        LOG.push(copy of this.name)

fn use_one() changes LOG:
    r = Res(name=String("a"))
    println(r.name)

fn main() changes LOG:
    use_one()
    println(LOG.len(), LOG[0])

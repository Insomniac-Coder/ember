#$ test: run-pass
#$ rules: FN-4
#$ stdout: a
#$ stdout: 1 a
## design rules: R5
## R5, verdicts 57, 86: a `drop` that pushes to `LOG` is written `fn drop():`, and `use_one`, whose
## local `r` ends with it and so calls `drop`, and `main`, which calls `use_one`, write nothing either:
## what each changes is worked out from the bodies.

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop():
        LOG.push(copy of this.name)

fn use_one():
    r = Res(name=String("a"))
    println(r.name)

fn main():
    use_one()
    println(LOG.len(), LOG[0])

#$ test: run-pass
#$ rules: R5
#$ stdout: 0 1
## R5, verdict 86: `fire` changes its object and `LOG`, and its first line is `fn fire():`; what it
## changes, and so what `main` changes by calling it, is worked out from the bodies.

static mut LOG: Array[String] = []

class Gun:
    ammo: int = 1

    fn fire():
        this.ammo -= 1
        LOG.push(String("bang"))

fn main():
    g = Gun()
    g.fire()
    println(g.ammo, LOG.len())

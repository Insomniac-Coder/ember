#$ test: run-pass
#$ rules: R5
#$ stdout: 0 1
## R5: a module variable, a type or a change list may stand beside `changes this`
## (`fn fire() changes this, LOG:`); `main` names `LOG`, which `fire` changes.

static mut LOG: Array[String] = []

class Gun:
    ammo: int = 1

    fn fire() changes this, LOG:
        this.ammo -= 1
        LOG.push(String("bang"))

fn main() changes LOG:
    g = Gun()
    g.fire()
    println(g.ammo, LOG.len())

#$ test: run-pass
#$ rules: R2, R5
#$ stdout: two
#$ 3
## R2, R5: the nearest program that is allowed: `swap_in` names what its ended objects' `drop` changes
## (verdict 57); the old `Res` ends at the second `=`, the new one when `swap_in` returns.

static mut LOG: Array[String] = []

class Res:
    name: String

    fn drop() changes LOG:
        LOG.push(String("ended"))

fn swap_in() changes LOG:
    a = Res(name=String("one"))
    a = Res(name=String("two"))
    println(a.name)

fn main() changes LOG:
    LOG.push(String("start"))
    swap_in()
    println(LOG.len())

#$ test: run-pass
#$ rules: R2, R1
#$ stdout: pushed
#$ end b
#$ end a
## R2, R1: a part handed over on its own leaves its owner partly handed over (verdict 78): when `p`
## ends it ends only what it still owns (`right`), and `kept` ends the `left` it was given; each `Res`
## ends once, the later local first (completeness gap G11). Verified by its output and a clean
## AddressSanitizer run (no double free).

class Res:
    name: String

    fn drop():
        println("end", this.name)

struct Pair:
    left: Res
    right: Res

fn main():
    kept: Array[Res] = []
    p = Pair(left=Res(name=String("a")), right=Res(name=String("b")))
    kept.push(p.left)
    println("pushed")

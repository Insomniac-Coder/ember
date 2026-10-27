#$ test: run-pass
#$ rules: R7, R8
#$ stdout: 30
## R7: a value that holds views, a local list of views included, lives no longer than what they point
## into, with nothing written; any number of `ref` views may read one place. AN-24's NEW program.

struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

    fn hp() -> int:
        return this.reg.hp[this.entity]

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    total = 0
    for v in vs:
        total += v.hp()
    println(total)

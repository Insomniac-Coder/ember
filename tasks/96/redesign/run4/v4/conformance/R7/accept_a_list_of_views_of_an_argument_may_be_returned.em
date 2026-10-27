#$ test: run-pass
#$ rules: R7, R6
#$ stdout: 30
## R7: the way out E3063's message gives: the registry is made in the caller, and the list of views
## returned with `from reg` lives as long as it. AN-24, point 6.

struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

fn views(reg: Registry) -> Array[View] from reg:
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    return vs

fn main():
    reg = Registry(hp=[10, 20])
    vs = views(reg)
    total = 0
    for v in vs:
        total += v.reg.hp[v.entity]
    println(total)

#$ test: run-pass
#$ rules: LT-45, BRW-16
#$ stdout: 10 3
## design rules: R7, R8
## R7, R8 (b): the way out E3060's message gives: the number is read again after the push made through
## the other holder. AN-24, point 6.

struct Registry:
    hp: Array[int]

struct Editor:
    reg: sharedref mut Registry
    entity: int

fn main():
    reg = Registry(hp=[10, 20])
    eds: Array[Editor] = []
    eds.push(Editor(reg=sharedref mut reg, entity=0))
    eds.push(Editor(reg=sharedref mut reg, entity=1))
    eds[1].reg.hp.push(5)
    println(eds[0].reg.hp[0], eds[0].reg.hp.len())

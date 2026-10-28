#$ test: run-pass
#$ rules: R7, R8
#$ stdout:
#$ 0
#$ 1
#$ 5
## R7, R8 (g): the first way out E3021's message gives: the registry changes below the loop, after the
## last use of the list of `ref` views. AN-24, point 6.

struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

fn main():
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    vs.push(View(reg=ref reg, entity=1))
    for v in vs:
        println(v.entity)
    reg.hp[0] = 5
    println(reg.hp[0])

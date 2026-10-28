#$ test: compile-fail
#$ rules: BRW-1
#$ help: reg: sharedref mut Registry
## design rules: R7, R8
## R7, R8 (g): a kept `ref` view reads; what it reads does not change through another name while the
## list that keeps it is used (E3021 at the change). AN-24, point 6.

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
    reg.hp[0] = 5    #$ error[E3021]: `reg.hp[0] = 5` (line 19) changes the registry that `vs[1].reg` reads (`View(reg=ref reg, entity=1)`, line 18), and `vs` is used on line 20; write `reg.hp[0] = 5` below the `for v in vs:` loop instead of on line 19, or, if the registry must change while the views are kept, write `reg: sharedref mut Registry` in `View` instead of `reg: ref Registry`, and make both views with it: `vs.push(View(reg=sharedref mut reg, entity=0))` and `vs.push(View(reg=sharedref mut reg, entity=1))` instead of the two `vs.push(View(reg=ref reg, ...))` lines
    for v in vs:
        println(v.entity)

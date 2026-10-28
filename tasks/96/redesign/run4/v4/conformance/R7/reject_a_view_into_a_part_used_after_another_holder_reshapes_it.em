#$ test: compile-fail
#$ rules: R7, R8
#$ help: println(eds[0].reg.hp[0])
## R7, R8 (b): every view into a `sharedref mut` target ends at a reshape made through any of its
## holders. AN-24, point 6. `main` changes the registry through a link, so it says `changes Registry`
## (R5, verdict 64).

struct Registry:
    hp: Array[int]

struct Editor:
    reg: sharedref mut Registry
    entity: int

fn main() changes Registry:
    reg = Registry(hp=[10, 20])
    eds: Array[Editor] = []
    eds.push(Editor(reg=sharedref mut reg, entity=0))
    eds.push(Editor(reg=sharedref mut reg, entity=1))
    s = eds[0].reg.hp.as_span()
    eds[1].reg.hp.push(5)
    println(s[0])    #$ error[E3060]: `eds[1].reg.hp.push(5)` (line 21) may move the list `s` points into (`eds[0].reg` and `eds[1].reg` may be the same registry), and `s` is used on line 22; use `s` before line 21, or read the number again after it: write `println(eds[0].reg.hp[0])` in place of line 22

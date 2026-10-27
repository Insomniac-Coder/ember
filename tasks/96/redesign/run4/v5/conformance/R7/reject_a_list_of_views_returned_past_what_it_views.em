#$ test: compile-fail
#$ rules: LT-45
#$ help: fn views(reg: Registry) -> Array[View] from reg:
## design rules: R7
## R7: a list of views lives no longer than what its views point into, whoever owns the list; returned
## past `reg` it is refused (E3063). AN-24, point 6 (its message, naming every line it replaces, R18).

struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

fn views() -> Array[View]:
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    return vs    #$ error[E3063]: `vs` holds views of `reg` (line 15), which ends when `views` returns, so it may not be returned; make the registry in the caller and give it to `views`: write `fn views(reg: Registry) -> Array[View] from reg:` in place of line 14 and remove line 15, and in `main` write `reg = Registry(hp=[10, 20])` above line 21 and `vs = views(reg)` in place of line 21

fn main():
    vs = views()
    println(vs.len())

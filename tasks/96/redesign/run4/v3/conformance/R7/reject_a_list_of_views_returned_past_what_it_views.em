#$ test: compile-fail
#$ rules: R7
#$ help: fn views(reg: Registry) -> Array[View] from reg:
## R7: a list of views lives no longer than what its views point into, whoever owns the list; returned
## past `reg` it is refused (E3063). AN-24, point 6.

struct Registry:
    hp: Array[int]

struct View:
    reg: ref Registry
    entity: int

fn views() -> Array[View]:
    reg = Registry(hp=[10, 20])
    vs: Array[View] = []
    vs.push(View(reg=ref reg, entity=0))
    return vs    #$ error[E3063]: `vs` holds views of `reg`, which ends when `views` returns, so it may not be returned; make the registry in the caller and give it to `views`: write `fn views(reg: Registry) -> Array[View] from reg:` without the line that makes `reg`, and call it `reg = Registry(hp=[10, 20])`, then `vs = views(reg)`

fn main():
    println(views().len())

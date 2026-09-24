#$ test: run-pass
#$ rules: CLS-10, CLS-2
#$ stdout: front 0 false 3
# `[CLS-10]` — a derived class with no `init` gets its base's constructor:
# `Door("front")` evaluates `Door`'s field defaults and then runs
# `Script.init("front")`.

open class Script:
    name: String
    ticks: int = 0

    fn init(mut self, owned name: String):
        self.name = name

class Door(Script):
    is_open: bool = false
    hits: int = 3

fn main():
    d = Door("front")
    println(d.name, d.ticks, d.is_open, d.hits)

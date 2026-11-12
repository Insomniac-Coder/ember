#$ test: compile-fail
#$ rules: CLS-3
# `[CLS-3]` — a derived `init` calls a memberwise base's constructor as `super.init(…)`, which takes
# every field that has no default.

open class Panel:
    name: String
    visible: bool = true

class Side(Panel):
    size: int = 3

    fn init():
        super.init(visible=false)       #$ error[E2021]: `super.init(visible=false)` gives nothing for `name`, which has no default: `Panel(name: String, visible: bool = …)`

fn main():
    s = Side()
    println(s.size)

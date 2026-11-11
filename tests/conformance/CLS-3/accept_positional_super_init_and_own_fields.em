#$ test: run-pass
#$ rules: CLS-3, CLS-11
#$ stdout: a false 7
# `[CLS-3]` — positional arguments in field order; the derived class's own fields are set before
# `super.init` as `[CLS-11]` asks.

open class Panel:
    name: String
    visible: bool = true

class Side(Panel):
    size: int

    fn init(owns name: String, size: int):
        this.size = size
        super.init(name, false)

fn main():
    s = Side(String("a"), 7)
    println(s.name, s.visible, s.size)

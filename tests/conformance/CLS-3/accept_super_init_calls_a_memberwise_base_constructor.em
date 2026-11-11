#$ test: run-pass
#$ rules: CLS-3, CLS-10, CLS-11
#$ stdout: side true 3
#$ stdout: side true 3
# `[CLS-3]` — a derived `init` calls a memberwise base's constructor as `super.init(…)`, with named
# arguments, a field with a default left out keeping the default; a derived class with no `init`
# gets it as its base's constructor (`[CLS-10]`).

open class Panel:
    name: String
    visible: bool = true

class Side(Panel):
    size: int = 3

    fn init(owns name: String):
        super.init(name=name)

class Plain(Panel):
    size: int = 3

fn main():
    s = Side(name=String("side"))
    println(s.name, s.visible, s.size)
    p = Plain(name=String("side"))
    println(p.name, p.visible, p.size)

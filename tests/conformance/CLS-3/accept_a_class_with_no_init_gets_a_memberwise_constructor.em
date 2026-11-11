#$ test: run-pass
#$ rules: CLS-3, STR-1
#$ stdout: side true 3
# `[CLS-3]` — a class with no `init` and no base class gets a memberwise constructor as a struct
# does: named arguments, a field with a default left out keeping the default.

class Panel:
    name: String
    visible: bool = true
    size: int

fn main():
    p = Panel(name=String("side"), size=3)
    println(p.name, p.visible, p.size)

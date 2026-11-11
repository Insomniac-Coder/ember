#$ test: run-pass
#$ rules: CLS-3, CLS-10
#$ stdout: side true
# `[CLS-3]`, `[CLS-10]` — a derived class with no `init` gets its base's memberwise constructor
# (`Panel(name=String("side"))`), a field with a default left out keeping the default.

open class Base:
    name: String
    visible: bool = true

class Panel(Base):
    pass

fn main():
    p = Panel(name=String("side"))
    println(p.name, p.visible)

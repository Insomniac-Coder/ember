#$ test: run-pass
#$ rules: CLS-3, CLS-10
#$ stdout: side true
# `[CLS-3]` — a derived class with no `init` passes its base's memberwise constructor on to a class
# derived from it.

open class Base:
    name: String
    visible: bool = true

open class Panel(Base):
    pass

class Side(Panel):
    pass

fn main():
    s = Side(name=String("side"))
    println(s.name, s.visible)

#$ test: run-pass
#$ rules: CLS-11, CLS-4
#$ stdout: derived t
#$ stdout: d
# `[CLS-11]` — a derived `init` assigns its own fields before `super.init`,
# so a `virtual` method the base `init` calls already sees them.

open class Base:
    name: String

    fn init(owns name: String):
        this.name = name
        this.describe()

    virtual fn describe():
        println("base")

class Derived(Base):
    tag: String

    fn init(owns tag: String):
        this.tag = tag
        super.init("d")
        println(this.name)

    override fn describe():
        println(f"derived {this.tag}")

fn main():
    _d = Derived("t")

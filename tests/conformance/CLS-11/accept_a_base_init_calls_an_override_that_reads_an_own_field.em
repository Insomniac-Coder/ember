#$ test: run-pass
#$ rules: CLS-11, CLS-4
#$ stdout: derived t
#$ stdout: d
# `[CLS-11]` — a derived `init` assigns its own fields before `super.init`,
# so a `virtual` method the base `init` calls already sees them.

open class Base:
    name: String

    fn init(self, owned name: String):
        self.name = name
        self.describe()

    virtual fn describe(self):
        println("base")

class Derived(Base):
    tag: String

    fn init(self, owned tag: String):
        self.tag = tag
        super.init("d")
        println(self.name)

    override fn describe(self):
        println(f"derived {self.tag}")

fn main():
    _d = Derived("t")

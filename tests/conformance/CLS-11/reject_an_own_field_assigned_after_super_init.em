#$ test: compile-fail
#$ rules: CLS-11
# D-564: the base `init` below reads `tag` through the override before the
# derived `init` assigns it.

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
        super.init("d") #$ error[E2102]: field `tag` is not assigned on every path before `super.init(...)`
        self.tag = tag

    override fn describe(self):
        println(f"derived {self.tag.len()}")

fn main():
    _d = Derived("t")

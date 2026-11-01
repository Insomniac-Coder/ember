#$ test: compile-fail
#$ rules: CLS-11
# D-564: the base `init` below reads `tag` through the override before the
# derived `init` assigns it.

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
        super.init(owns "d") #$ error[E2102]: field `tag` is not assigned on every path before `super.init(...)`
        this.tag = tag

    override fn describe():
        println(f"derived {this.tag.len()}")

fn main():
    _d = Derived("t")

#$ test: run-pass
#$ rules: DSP-1, DSP-2, CLS-4
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ stdout: 2
#$ stdout: 3
#$ stdout: 5
#$ assert-c-count: contains("->ti->vtable))->slot") == 2
# `[DSP-1]` — a call on a final class's handle is dispatched statically, an
# inherited method's too: `Child` can be nothing but a `Child`, so
# `child.label()` calls `Base.label` directly although `Other` replaces it.
# The two calls through the open base read the object's table and run the
# overrides.

open class Base:
    virtual fn tick(self) -> i32:
        return 1

    virtual fn label(self) -> i32:
        return 3

    fn init(mut self):
        pass

class Child(Base):
    override fn tick(self) -> i32:
        return 2

    fn init(mut self):
        super.init()

class Other(Base):
    override fn label(self) -> i32:
        return 5

    fn init(mut self):
        super.init()

fn main():
    child = Child()
    base: Base = child
    other: Base = Other()
    println(base.tick())
    println(child.tick())
    println(child.label())
    println(other.label())

#$ test: run-pass
#$ rules: EXC-7, MAN-3, TST-11
#$ profiles: debug, release, shipping
#$ warning[L3013]: long-term access to `self` is live across this call
#$ stdout: 7

# With the opt-in lint enabled, a mutable class method's long-term access stays
# live across a virtual call in the same open hierarchy. The program is safe
# and still runs; L3013 is advisory rather than an error.
open class Base:
    value: i32

    fn init(mut self):
        self.value = 7

    virtual fn ping(self) -> i32:
        return self.value

    virtual fn relay(mut self) -> i32:
        return self.ping()

class Derived(Base):
    fn init(mut self):
        super.init()

    override fn ping(self) -> i32:
        return self.value

fn main():
    derived = Derived()
    base: Base = derived
    println(base.relay())

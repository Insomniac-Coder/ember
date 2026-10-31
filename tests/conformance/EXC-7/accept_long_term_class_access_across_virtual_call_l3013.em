#$ test: run-pass
#$ rules: EXC-7, MAN-3, TST-11
#$ profiles: debug, development, release
#$ warning[L3013]: long-term access to `this` is live across this call
#$ stdout: 8

# With the opt-in lint enabled, the long-term access of a class method that changes its object
# (`relay` counts its calls in `this.value`) stays live across a virtual call in the same open
# hierarchy. The program is safe
# and still runs; L3013 is advisory rather than an error.
open class Base:
    value: i32

    fn init():
        this.value = 7

    virtual fn ping() -> i32:
        return this.value

    virtual fn relay() -> i32:
        this.value += 1
        return this.ping()

class Derived(Base):
    fn init():
        super.init()

    override fn ping() -> i32:
        return this.value

fn main():
    derived = Derived()
    base: Base = derived
    println(base.relay())

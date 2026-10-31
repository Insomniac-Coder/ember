#$ test: run-pass
#$ rules: EXC-7, MAN-3, TST-11, TYP-22
#$ profiles: debug, development, release
#$ warning[L3013]: long-term access to `this` is live across this call
#$ stdout: 7

# The opt-in long-term-access lint also covers a mutable erased receiver. A
# `ref mut dyn` carrier is derived from the active class receiver, so the call
# can re-enter the open hierarchy while the method-duration access is live.
interface Probe:
    fn inspect() -> i32

open class Base implements Probe:
    value: i32

    fn init():
        this.value = 7

    fn inspect() -> i32:
        return this.value

    virtual fn relay() -> i32:
        probe: ref mut dyn Probe = ref mut this
        return probe.inspect()

class Derived(Base):
    fn init():
        super.init()

fn main():
    derived = Derived()
    base: Base = derived
    println(base.relay())

#$ test: run-pass
#$ rules: EXC-7, MAN-3, TST-11, TYP-22
#$ profiles: debug, development, release
#$ warning[L3013]: long-term access to `this` is live across this call
#$ stdout: 8

# The same opt-in warning applies when the nested call crosses the erased
# `ref dyn` boundary. Its receiver is derived from the active class receiver,
# so the call can re-enter this open-class hierarchy.
interface Probe:
    fn inspect() -> i32

open class Base implements Probe:
    value: i32

    fn init():
        this.value = 7

    fn inspect() -> i32:
        return this.value

    virtual fn relay() -> i32:
        this.value += 1
        probe: ref dyn Probe = ref this
        return probe.inspect()

class Derived(Base):
    fn init():
        super.init()

fn main():
    derived = Derived()
    base: Base = derived
    println(base.relay())

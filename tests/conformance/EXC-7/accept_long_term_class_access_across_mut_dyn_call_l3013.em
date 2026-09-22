#$ test: run-pass
#$ rules: EXC-7, MAN-3, TST-11, TYP-22
#$ profiles: debug, release, shipping
#$ warning[L3013]: long-term access to `self` is live across this call
#$ stdout: 7

# The opt-in long-term-access lint also covers a mutable erased receiver. A
# `ref mut dyn` carrier is derived from the active class receiver, so the call
# can re-enter the open hierarchy while the method-duration access is live.
interface Probe:
    fn inspect(self) -> i32

open class Base implements Probe:
    value: i32

    fn init(mut self):
        self.value = 7

    fn inspect(self) -> i32:
        return self.value

    virtual fn relay(mut self) -> i32:
        probe: ref mut dyn Probe = ref mut self
        return probe.inspect()

class Derived(Base):
    fn init(mut self):
        super.init()

fn main():
    derived = Derived()
    base: Base = derived
    println(base.relay())

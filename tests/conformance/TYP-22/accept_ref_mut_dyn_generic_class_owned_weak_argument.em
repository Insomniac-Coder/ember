#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("em_vt_dyn_7Observe_for_6ReaderGda5cc12a188c7619_slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# An owned Weak[Shared[Token]] argument crossing a ref mut dyn carrier is
# copied for the generic class implementation. The mutable dynamic receiver
# closes its write interval while the callee upgrades and destroys its copy.
interface Observe:
    fn observe(owns weak: Weak[Shared[Token]]) -> i32

struct Token:
    value: i32

[T]
class Reader implements Observe:
    marker: T
    count: i32

    fn observe(owns weak: Weak[Shared[Token]]) -> i32:
        this.count = this.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn inspect(ref mut value: dyn Observe, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(owns weak)

fn main():
    strong = Shared(Token(7))
    reader = Reader[bool](true, 0)
    weak = Weak(strong)
    println(inspect(ref mut reader, weak))

#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 0
#$ assert-c: contains("em_vt_dyn_Observe_Reader_bool_slot0")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# An owned Weak[Shared[Token]] argument crossing a ref mut dyn carrier remains
# only a control-block observer after expiry. The mutable generic class closes
# its write interval while upgrade() takes None and the owned copy is dropped.
interface Observe:
    fn observe(mut self, owned weak: Weak[Shared[Token]]) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe:
    marker: T
    count: i32

    fn observe(mut self, owned weak: Weak[Shared[Token]]) -> i32:
        self.count = self.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn inspect(value: ref mut dyn Observe, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak)

fn main():
    reader = Reader[bool](true, 0)
    weak = expired()
    println(inspect(ref mut reader, weak))

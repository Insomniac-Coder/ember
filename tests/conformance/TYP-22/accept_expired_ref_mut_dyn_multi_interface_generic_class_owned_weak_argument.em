#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, OWN-2, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# An owned weak argument through ref mut Observe + Bump gets an independent
# observer even after its target expires. The mutable slot updates the reader,
# drops its argument copy, and the second slot still returns the new count.
interface Observe:
    fn observe(owns weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump() -> i32

struct Token:
    value: i32

[T]
class Reader implements Observe, Bump:
    marker: T
    count: i32

    fn observe(owns weak: Weak[Shared[Token]]) -> i32:
        this.count = this.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump() -> i32:
        return this.count

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn inspect(ref mut value: dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(owns weak) + value.bump()

fn main():
    reader = Reader[bool](true, 0)
    println(inspect(ref mut reader, expired()))

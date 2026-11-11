#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, DRP-6, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 0
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6ReaderGda5cc12a188c7619b25d920e0657739a_slot1")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")
#$ assert-c: !contains("ember_box_new_copy")

# A borrowed Weak[Shared[Token]] argument through a composed dynamic box stays
# a valid observer after its target expires. The generic implementation must
# take the None path while the second interface slot remains callable.
interface Observe:
    fn observe(weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump() -> i32

struct Token:
    value: i32

[T]
class Reader implements Observe, Bump:
    marker: T

    fn observe(weak: Weak[Shared[Token]]) -> i32:
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump() -> i32:
        return 1

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn main():
    boxed: Box[dyn Observe + Bump] = Box(Reader[bool](true))
    println(boxed.observe(expired()))
    println(boxed.bump())

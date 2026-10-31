#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-2, CLS-4, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ stdout: end
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot0")
#$ assert-c: contains("em_vt_dyn_7Observe_4Bump_for_6HolderGda5cc12a188c7619_slot1")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A generic derived class with inherited state accepts a borrowed weak argument
# through ref mut Observe + Bump. The mutable slot updates the base counter and
# the second slot observes the same payload.
interface Observe:
    fn observe(weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump() -> i32

struct Token:
    value: i32

[T]
open class Base:
    marker: T
    count: i32

    fn init(marker: T):
        this.marker = marker
        this.count = 0

[T]
class Holder(Base[T]) implements Observe, Bump:
    fn init(marker: T):
        super.init(marker)

    fn observe(weak: Weak[Shared[Token]]) -> i32:
        this.count = this.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump() -> i32:
        return this.count

fn inspect(ref mut value: dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak) + value.bump()

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
    strong = Shared(Token(7))
    holder = Holder[bool](true)
    println(inspect(ref mut holder, Weak(strong)))

fn main():
    program()
    println("end")

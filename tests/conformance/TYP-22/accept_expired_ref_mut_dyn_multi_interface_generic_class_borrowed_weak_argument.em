#$ test: run-pass
#$ rules: TYP-16, TYP-22, IFC-1, CLS-4, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot0")
#$ assert-c: contains("em_vt_dyn_multi__Observe__Bump_Reader_bool_slot1")
#$ assert-c: contains("ember_access_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A borrowed weak argument crossing a ref mut composed carrier takes None after
# expiry. The mutable generic class still updates its count for the second
# interface slot.
interface Observe:
    fn observe(mut self, weak: Weak[Shared[Token]]) -> i32

interface Bump:
    fn bump(mut self) -> i32

struct Token:
    value: i32

class Reader[T] implements Observe, Bump:
    marker: T
    count: i32

    fn observe(mut self, weak: Weak[Shared[Token]]) -> i32:
        self.count = self.count + 1
        match weak.upgrade():
            Some(owner):
                return owner.get().value
            None:
                return 0

    fn bump(mut self) -> i32:
        return self.count

fn expired() -> Weak[Shared[Token]]:
    strong = Shared(Token(7))
    return Weak(strong)

fn inspect(value: ref mut dyn Observe + Bump, weak: Weak[Shared[Token]]) -> i32:
    return value.observe(weak) + value.bump()

fn main():
    reader = Reader[bool](true, 0)
    println(inspect(ref mut reader, expired()))

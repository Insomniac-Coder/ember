#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ stdout: 5
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# Replacing a live inherited Weak[Shared[Token]] field through a generic
# mutable virtual call releases only the field observer. An outside observer
# to the old Shared payload stays valid while the override upgrades its new
# replacement.
struct Token:
    value: i32

[T]
open class Base:
    weak: Weak[Shared[Token]]
    marker: T
    count: i32

    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        this.weak = weak
        this.marker = marker
        this.count = count

    virtual fn replace(replacement: Weak[Shared[Token]]) -> i32:
        this.weak = replacement
        this.count = this.count + 1
        return this.count

[T]
class Derived(Base[T]):
    fn init(weak: Weak[Shared[Token]], marker: T, count: i32):
        super.init(weak, marker, count)

    override fn replace(replacement: Weak[Shared[Token]]) -> i32:
        this.weak = replacement
        this.count = this.count + 1
        match this.weak.upgrade():
            Some(owner):
                return owner.get().value + this.count
            None:
                return this.count

fn main():
    previous = Shared(Token(5))
    replacement = Shared(Token(7))
    old = Weak(previous)
    derived = Derived[bool](old, true, 0)
    base: Base[bool] = derived
    println(base.replace(Weak(replacement)))
    match old.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)

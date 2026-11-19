#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, BRW-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 8
#$ assert-c: contains("static int32_t em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A mutable generic virtual call replaces an inherited weakref Token
# field whose prior observer is already expired. The assignment must release
# only that old observer, retain the replacement, and upgrade the new target.
struct Token:
    value: i32

[T]
open class Base:
    weak: weakref Token
    marker: T
    count: i32

    fn init(weak: weakref Token, marker: T, count: i32):
        this.weak = weakref weak
        this.marker = marker
        this.count = count

    virtual fn replace(replacement: weakref Token) -> i32:
        this.weak = weakref replacement
        this.count = this.count + 1
        return this.count

[T]
class Derived(Base[T]):
    fn init(weak: weakref Token, marker: T, count: i32):
        super.init(weak, marker, count)

    override fn replace(replacement: weakref Token) -> i32:
        this.weak = weakref replacement
        this.count = this.count + 1
        if this.weak exists as owner:
            return owner.value + this.count
        else:
            return this.count

fn expired() -> weakref Token:
    strong = shares mut Token(5)
    return weakref strong

fn main():
    replacement = shares mut Token(7)
    derived = Derived[bool](expired(), true, 0)
    base: Base[bool] = derived
    println(base.replace(weakref replacement))

#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, EXC-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, WK-14, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static ember_obj_header* em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_object_begin_write")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A mutable generic virtual call may return weakref Token from an
# inherited field after updating derived state, without keeping the final
# `shares` owner alive. Once the helper exits, the returned observer must use the
# expired upgrade path.
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

    virtual fn observer() -> weakref Token:
        this.count = this.count + 1
        return weakref this.weak

[T]
class Derived(Base[T]):
    fn init(weak: weakref Token, marker: T, count: i32):
        super.init(weak, marker, count)

    override fn observer() -> weakref Token:
        this.count = this.count + 2
        return weakref this.weak

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    derived = Derived[bool](weakref strong, true, 0)
    base: Base[bool] = derived
    return base.observer()

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)

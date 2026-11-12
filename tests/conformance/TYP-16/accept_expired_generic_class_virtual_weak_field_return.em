#$ test: run-pass
#$ rules: TYP-16, CLS-2, CLS-4, DSP-1, HEAP-3, HEAP-4, HEAP-5, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, development, release
#$ stdout: 7
#$ assert-c: contains("static ember_obj_header* em_vt_7DerivedGda5cc12a188c7619b25d920e0657739a_slot0")
#$ assert-c: contains("->slot0")
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A weak observer returned through a generic virtual slot outlives the derived
# object and its final `shares` owner. It keeps the control block only, so the
# later upgrade must report None instead of resurrecting Token.
struct Token:
    value: i32

[T]
open class Base:
    weak: weakref Token
    marker: T

    fn init(weak: weakref Token, marker: T):
        this.weak = weakref weak
        this.marker = marker

    virtual fn observer() -> weakref Token:
        return weakref this.weak

[T]
class Derived(Base[T]):
    fn init(weak: weakref Token, marker: T):
        super.init(weak, marker)

    override fn observer() -> weakref Token:
        return weakref this.weak

fn expired() -> weakref Token:
    strong = shares mut Token(7)
    derived = Derived[bool](weakref strong, true)
    base: Base[bool] = derived
    return base.observer()

fn main():
    weak = expired()
    if weak exists as _:
        println(0)
    else:
        println(7)

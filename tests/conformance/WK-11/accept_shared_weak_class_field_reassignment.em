#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-6, HEAP-7, WK-11, WK-12, WK-13, TST-26
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ assert-c: contains("ember_weak_retain(")
#$ assert-c: contains("ember_weak_release(")
#$ assert-c: contains("ember_weak_upgrade(")

# A class field may replace an empty Weak[Shared[T]] value with an observer of
# a live Shared allocation. The field owns only a weak count; payload access
# after upgrade still crosses the explicit Shared.get() boundary.
struct Token:
    value: i32

class Holder:
    weak: Weak[Shared[Token]] = Weak[Shared[Token]].empty()

fn main():
    strong = Shared(Token(42))
    holder = Holder()
    holder.weak = Weak(strong)
    match holder.weak.upgrade():
        Some(owner):
            println(owner.get().value)
        None:
            println(0)

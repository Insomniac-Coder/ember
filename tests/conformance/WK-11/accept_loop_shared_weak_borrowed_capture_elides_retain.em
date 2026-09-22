#$ test: run-pass
#$ rules: HEAP-3, HEAP-4, HEAP-7, HEAP-6, CTL-1, CTL-2, WK-11, WK-12, WK-13, TYP-14, CLO-1, CLO-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 7

# A normal closure keeps the loop's borrowed Weak[Shared[T]] handle by
# reference. `upgrade` must see the same Shared allocation without a third weak
# retain, and the resulting Shared owner still requires explicit `get()`.
struct Token:
    value: i32

fn main():
    owner = Shared(Token(7))
    weak: Weak[Shared[Token]] = Weak(owner)
    copies: Array[Weak[Shared[Token]]] = Array[Weak[Shared[Token]]]()
    copies.push(weak)
    for observed in copies:
        task = fn() -> i32:
            match observed.upgrade():
                Some(live):
                    value = live.get()
                    return value.value
                None:
                    return 0
        println(task())

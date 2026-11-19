#$ test: run-pass
#$ rules: CTL-1, CTL-2, WK-11, WK-12, TYP-14, CLO-1, CLO-2
#$ profiles: debug, development, release
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 7
#$ stdout: end

# A normal closure keeps the loop's borrowed `weakref` handle by reference. Opening
# it through that reference must reach the link without creating another weak
# owner.
class Token:
    value: i32

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
    token = Token(7)
    weak: weakref Token = weakref token
    copies: Array[weakref Token] = Array[weakref Token]()
    copies.push(owns (weakref weak))
    for observed in copies:
        fn task_fn() -> i32:
            if observed exists as owner:
                return owner.value
            return 0
        task = task_fn
        println(task())

fn main():
    program()
    println("end")

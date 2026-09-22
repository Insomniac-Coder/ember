#$ test: run-pass
#$ rules: CTL-1, CTL-2, WK-11, WK-12, TYP-14, CLO-1, CLO-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 7

# A normal closure keeps the loop's borrowed Weak handle by reference. Calling
# upgrade through that reference must auto-dereference the receiver without
# creating another weak owner.
class Token:
    value: i32

fn main():
    token = Token(7)
    weak: Weak[Token] = Weak(token)
    copies: Array[Weak[Token]] = Array[Weak[Token]]()
    copies.push(weak)
    for observed in copies:
        task = fn() -> i32:
            match observed.upgrade():
                Some(owner):
                    return owner.value
                None:
                    return 0
        println(task())

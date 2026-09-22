#$ test: run-pass
#$ rules: CTL-1, CTL-2, WK-11, WK-12
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 7

# A loop over `Array[Weak[Token]]` borrows the Weak handle stored in the
# Array. Construction and insertion retain the weak count; iteration itself
# must not manufacture another weak owner.
class Token:
    value: i32

fn main():
    token = Token(7)
    weak: Weak[Token] = Weak(token)
    copies: Array[Weak[Token]] = Array[Weak[Token]]()
    copies.push(weak)
    for observed in copies:
        match observed.upgrade():
            Some(owner):
                println(owner.value)
            None:
                println(0)

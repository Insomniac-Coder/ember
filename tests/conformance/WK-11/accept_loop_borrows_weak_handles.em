#$ test: run-pass
#$ rules: CTL-1, CTL-2, WK-11, WK-12
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_weak_retain(") == 2
#$ assert-c-count: contains("ember_weak_release(") == 2
#$ stdout: 7
#$ stdout: end

# A loop over `Array[Weak[Token]]` borrows the Weak handle stored in the
# Array. Construction and insertion retain the weak count; iteration itself
# must not manufacture another weak owner.
class Token:
    value: i32

# `program` holds the test, so its values die before `main`'s last
# statement: at the end of `main` a release build leaves what only frees
# memory to the operating system (`[PHIL-5]`), and the drops read here
# would go with it.
fn program():
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

fn main():
    program()
    println("end")

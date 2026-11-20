# Clause CLO-10.2: "A named function, or a lambda that captures nothing, has no state and allocates
# nothing, and is `Copy`; a function value with state is not, and `=` hands it over"; "A call through
# a list element or a field copies the two words out"; "Replacing or ending a function value ends
# what its state owns, as ending an object runs its `drop`"; "two are equal (`==`) when they call the
# same function with the same state"
#$ test: run-pass
#$ rules: CLO-10, DRP-8, CLO-1
#$ stdout: 14
#$ stdout: a
#$ stdout: a ends
#$ stdout: after
#$ stdout: true
#$ stdout: 2
# A stateless function copied and both used; a stateful one replaced, which ends what it owned; two
# copies of one stateless function compared; a call through a list element.

class Res:
    name: String

    fn init(owns name: String):
        this.name = name

    fn drop():
        println(f"{this.name} ends")

fn seven() -> int:
    return 7

fn main():
    f = seven
    g = f
    println(f() + g())
    r = Res(String("a"))
    h: fn() = fn() owns r: println(r.name)
    h()
    h = fn(): pass
    println("after")
    println(f == g)
    fs: Array[fn() -> int] = [fn() => 2]
    println(fs[0]())

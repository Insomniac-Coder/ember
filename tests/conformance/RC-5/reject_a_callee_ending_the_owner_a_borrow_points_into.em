#$ test: compile-fail
#$ rules: RC-5
# ODR-276's question, with the tests: can a callee end the object a borrowed parameter points
# into? Not in safe code (`[RC-5]`, `[EXC-2]`); see the comment on the function below.

class Child:
    v: int
    name: String

    fn init(v: int, owns name: String):
        this.v = v
        this.name = name

    fn drop():
        println("drop", this.name)

## `[RC-5]` — no call in a view's life may end the object it views: handing `c` over beside a view into it is refused.
fn take(ref r: int, owns c: Child) -> int:
    println("dropping")
    mem.drop(c)
    println("read", r)
    return r

fn main():
    c = Child(5, "five".to_string())
    println(take(ref c.v, owns c))    #$ error[E3021]: cannot be moved while it is borrowed

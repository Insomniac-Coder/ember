#$ test: run-pass
#$ rules: RC-5
#$ profiles: debug, development, release
#$ stdout: dropping
#$ stdout: read 5
#$ stdout: 5
#$ stdout: drop five
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

## The callee ends the owner it was given while its other parameter borrows into the owner's
## object: the caller's loan keeps the object alive (`[RC-5]`) to the end of its statement.
fn take(r: ref int, owns c: Child) -> int:
    println("dropping")
    mem.drop(c)
    println("read", r)
    return r

fn main():
    c = Child(5, "five".to_string())
    println(take(ref c.v, owns c))

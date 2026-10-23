#$ test: run-pass
#$ rules: RC-5
#$ profiles: debug, development, release
#$ stdout: before 5
#$ stdout: replaced
#$ stdout: still 5
#$ stdout: drop five
#$ stdout: after 5
#$ stdout: end
#$ stdout: drop six
# ODR-276's question, with the tests: can a callee end the object a borrowed parameter points
# into? Not in safe code (`[RC-5]`, `[EXC-2]`); see the comment on the function below.

class Child:
    v: int
    name: String

    fn init(mut self, v: int, owned name: String):
        self.v = v
        self.name = name

    fn drop(mut self):
        println("drop", self.name)

class Parent:
    child: Child

    fn init(mut self, owned child: Child):
        self.child = child

## The callee replaces the only field holding the object its parameter points into: the caller's
## borrow goes through a retained copy of the handle (`[RC-5]`), so the object outlives the call.
fn read_then_replace(x: ref int, p: Parent) -> int:
    println("before", x)
    p.child = Child(6, "six".to_string())
    println("replaced")
    println("still", x)
    return x

fn main():
    p = Parent(Child(5, "five".to_string()))
    r = read_then_replace(ref p.child.v, p)
    println("after", r)
    println("end")

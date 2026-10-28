#$ test: run-fail
#$ rules: R14
#$ stdout: 3
#$ panics: invalid downcast
## R14: "`cast s as Circle` gives the `Circle` and stops the program if `s` is not one"; one type-id
## compare per cast (section 2). The rules give no text for the stop: this is today's
## (`compiler/ember_codegen_c/src/lib.rs`, `AssertKind::Downcast`).

interface Shape:
    fn size() -> int

struct Circle implements Shape:
    r: int

    fn size() -> int:
        return this.r

struct Square implements Shape:
    side: int

    fn size() -> int:
        return this.side

fn main():
    a: owns dyn Shape = Circle(r=3)
    c = cast a as Circle
    println(c.r)
    b: owns dyn Shape = Square(side=4)
    q = cast b as Circle
    println(q.r)

#$ test: run-pass
#$ rules: TYP-13, HEAP-1
#$ profiles: debug, release, shipping
#$ stdout: 8 8
#$ stdout: 7
#$ stdout: none
# Box[int] is a non-null pointer; None fits in its null value.
fn main():
    println(mem.size_of[Option[Box[int]]](), mem.size_of[Box[int]]())
    present: Option[Box[int]] = Some(Box(7))
    match present:
        Some(boxed):
            println(boxed.get())
        None:
            println("unexpected")
    absent: Option[Box[int]] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

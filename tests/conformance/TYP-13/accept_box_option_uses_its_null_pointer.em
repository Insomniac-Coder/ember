#$ test: run-pass
#$ rules: TYP-13, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 8 8
#$ stdout: 7
#$ stdout: none
# Box[int] is a non-null pointer; None fits in its null value.
fn main():
    println(mem.size_of[Option[owns int]](), mem.size_of[owns int]())
    present: Option[owns int] = Some(owns 7)
    match present:
        Some(boxed):
            println(boxed)
        None:
            println("unexpected")
    absent: Option[owns int] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

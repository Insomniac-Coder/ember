#$ test: run-pass
#$ rules: TYP-13, BRW-6
#$ profiles: debug, release, shipping
#$ stdout: 8 8
#$ stdout: 11
#$ stdout: none
# A reference cannot be null, so Option[ref int] is one pointer.
fn main():
    value = 11
    println(mem.size_of[Option[ref int]](), mem.size_of[ref int]())
    present: Option[ref int] = Some(ref value)
    match present:
        Some(view):
            println(view)
        None:
            println("unexpected")
    absent: Option[ref int] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

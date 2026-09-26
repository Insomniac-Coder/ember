#$ test: run-pass
#$ rules: TYP-13, SPN-1
#$ profiles: debug, release, shipping
#$ stdout: 16 16
#$ stdout: 0
#$ stdout: 2
#$ stdout: none
# An empty Span is still Some; None uses an impossible pointer/length pair.
fn main():
    empty: Array[int] = []
    items = [1, 2]
    println(mem.size_of[Option[Span[int]]](), mem.size_of[Span[int]]())
    first: Option[Span[int]] = Some(empty.as_span())
    match first:
        Some(view):
            println(view.len())
        None:
            println("unexpected")
    second: Option[Span[int]] = Some(items.as_span())
    match second:
        Some(view):
            println(view[1])
        None:
            println("unexpected")
    absent: Option[Span[int]] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

#$ test: run-pass
#$ rules: TYP-13, TXT-1
#$ profiles: debug, release, shipping
#$ stdout: 16 16
#$ stdout: true
#$ stdout: ok
#$ stdout: none
# Empty text remains Some even when its backing String has no buffer.
fn main():
    empty = String()
    println(mem.size_of[Option[str]](), mem.size_of[str]())
    first: Option[str] = Some(empty.as_str())
    match first:
        Some(view):
            println(view == "")
        None:
            println("unexpected")
    second: Option[str] = Some("ok")
    match second:
        Some(view):
            println(view)
        None:
            println("unexpected")
    absent: Option[str] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

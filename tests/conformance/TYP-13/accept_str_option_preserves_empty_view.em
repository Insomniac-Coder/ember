#$ test: run-pass
#$ rules: TYP-13, TXT-1
#$ profiles: debug, development, release
#$ stdout: 16 16
#$ stdout: true
#$ stdout: ok
#$ stdout: none
# Empty text remains Some even when its backing String has no buffer.
fn main():
    empty = String()
    println(mem.size_of[Option[StrView]](), mem.size_of[StrView]())
    first: Option[StrView] = Some(empty.as_str())
    match first:
        Some(view):
            println(view == "")
        None:
            println("unexpected")
    second: Option[StrView] = Some("ok")
    match second:
        Some(view):
            println(view)
        None:
            println("unexpected")
    absent: Option[StrView] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

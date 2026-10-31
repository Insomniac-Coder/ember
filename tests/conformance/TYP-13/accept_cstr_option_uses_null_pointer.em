#$ test: run-pass
#$ rules: TYP-13, FFI-15
#$ profiles: debug, development, release
#$ stdout: 8 8
#$ stdout: present
#$ stdout: none

fn main():
    println(mem.size_of[Option[CStrView]](), mem.size_of[CStrView]())
    present: Option[CStrView] = Some(c"present")
    match present:
        Some(_):
            println("present")
        None:
            println("unexpected")
    absent: Option[CStrView] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

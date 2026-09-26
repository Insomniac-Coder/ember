#$ test: run-pass
#$ rules: TYP-13, FFI-15
#$ profiles: debug, release, shipping
#$ stdout: 8 8
#$ stdout: present
#$ stdout: none

fn main():
    println(mem.size_of[Option[cstr]](), mem.size_of[cstr]())
    present: Option[cstr] = Some(c"present")
    match present:
        Some(_):
            println("present")
        None:
            println("unexpected")
    absent: Option[cstr] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

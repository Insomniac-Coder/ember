#$ test: run-pass
#$ rules: TYP-13, TYP-1
#$ profiles: debug, release, shipping
#$ stdout: 4 4
#$ stdout: A
#$ stdout: none
# A valid char cannot be U+110000, leaving that value for None.
fn main():
    println(mem.size_of[Option[char]](), mem.size_of[char]())
    present: Option[char] = Some('A')
    match present:
        Some(value):
            println(value)
        None:
            println("unexpected")
    absent: Option[char] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

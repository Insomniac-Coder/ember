#$ test: run-pass
#$ rules: RNG-7, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: 2 1
#$ stdout: some none
# A range covering every u8 value has no spare value for None.
type Full = u8 in 0 ..= 255

fn main():
    println(mem.size_of[Option[Full]](), mem.size_of[Full]())
    present: Option[Full] = Some(Full.clamped(255))
    absent: Option[Full] = None
    match present:
        Some(_):
            print("some ")
        None:
            print("unexpected ")
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

#$ test: run-pass
#$ rules: RNG-7, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: 1 1
#$ stdout: 40
#$ stdout: none
# Percent excludes values above 100, leaving one for Option's None.
type Percent = u8 in 0 ..= 100

fn main():
    println(mem.size_of[Option[Percent]](), mem.size_of[Percent]())
    present: Option[Percent] = Some(Percent.clamped(40))
    match present:
        Some(value):
            raw: u32 = value
            println(raw)
        None:
            println("unexpected")
    absent: Option[Percent] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

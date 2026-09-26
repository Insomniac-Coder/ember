#$ test: run-pass
#$ rules: RNG-7, TYP-13
#$ profiles: debug, release, shipping
#$ stdout: 4 4
#$ stdout: 0.5
#$ stdout: none
# A range value is never NaN, so NaN represents None.
type Fraction = f32 in 0.0 ..= 1.0

fn main():
    println(mem.size_of[Option[Fraction]](), mem.size_of[Fraction]())
    present: Option[Fraction] = Some(Fraction.clamped(0.5))
    match present:
        Some(value):
            raw: f32 = value
            println(raw)
        None:
            println("unexpected")
    absent: Option[Fraction] = None
    match absent:
        Some(_):
            println("unexpected")
        None:
            println("none")

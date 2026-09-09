#$ test: run-pass
#$ rules: RNG-1, LEX-15a
## An alias with an `in` clause declares a nominal numeric type over the named
## representation; one without it is transparent.

type Roughness = f32 in 0.0 ..= 1.0
type Percent   = u8  in 0 ..= 100
type Plain     = f32

fn main():
    r: Roughness = 0.5
    p: Percent = 40
    q: Plain = 2.0
    a: f32 = r
    b: u32 = p
    print(a)
    print(b)
    print(q)
#$ stdout: 0.5402

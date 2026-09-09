#$ test: run-pass
#$ rules: RNG-3a
## "On the C backend it lowers to two compares or the target's `min`/`max`
## instruction pair."

type Percent = u8 in 0 ..= 100

fn main():
    a: u32 = Percent.clamped(200)
    b: u32 = Percent.clamped(7)
    print(a)
    print(b)
#$ stdout: 1007

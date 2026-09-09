#$ test: run-pass
#$ rules: RNG-10, RNG-3, RNG-3a
## "The construction set is closed": a constant the compiler placed in range,
## `T.checked(v)`, `T.clamped(v)`, and a copy of an already-valid value.

type Percent = u8 in 0 ..= 100

fn main():
    a: Percent = 10                      # (a) a constant in range
    b = Percent.clamped(200)             # (c) the total form
    c = a                                # (e) a copy
    x: u32 = a
    y: u32 = b
    z: u32 = c
    print(x)
    print(y)
    print(z)
    match Percent.checked(50):           # (b) the fallible form
        Ok(p) => print(1)
        Err(e) => print(0)
#$ stdout: 10100101

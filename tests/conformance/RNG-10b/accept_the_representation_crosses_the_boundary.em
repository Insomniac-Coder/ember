#$ test: run-pass
#$ rules: RNG-10b
#$ stdout: 5
## `[RNG-10b]` — a range type may not appear in a foreign signature; the foreign side uses the
## representation, and a value arriving from it becomes a range value only through `checked`.

type Roughness = u8 in 0 ..= 100

pub extern "C" fn shade(r: u8) -> i32:
    match Roughness.checked(r):
        Ok(_):
            return 1
        Err(_):
            return 0

fn main():
    total = shade(5) + shade(100) + shade(101) + shade(250) + shade(0) + shade(60) + shade(99)
    println(total)

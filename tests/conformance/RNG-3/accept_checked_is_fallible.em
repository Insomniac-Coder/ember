#$ test: run-pass
#$ rules: RNG-3
## `T.checked(v) -> Result[T, RangeError]`.

type Roughness = f32 in 0.25 ..= 0.75

fn main():
    match Roughness.checked(2.0):
        Ok(g) => print(1)
        Err(e) => print(0)
    match Roughness.checked(0.5):
        Ok(g) => print(1)
        Err(e) => print(0)
#$ stdout: 01

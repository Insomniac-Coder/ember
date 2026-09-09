#$ test: run-pass
#$ rules: RNG-2
## A value of a range type is assignable to the same range type.

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    s: Roughness = r
    v: f32 = s
    print(v)
#$ stdout: 0.5

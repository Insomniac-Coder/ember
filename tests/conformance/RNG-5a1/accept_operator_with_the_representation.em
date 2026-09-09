#$ test: run-pass
#$ rules: RNG-5a1
## The three generated impls: `T op T`, `T op R`, `R op T`, each yielding `R`.

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    a: f32 = r - r
    b: f32 = r / 2.0
    c: f32 = 1.0 - r
    print(a)
    print(b)
    print(c)
#$ stdout: 00.250.5

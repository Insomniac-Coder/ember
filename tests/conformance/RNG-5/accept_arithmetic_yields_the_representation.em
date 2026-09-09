#$ test: run-pass
#$ rules: RNG-5
## "Arithmetic involving a range type normally yields its representation":
## `Roughness + Roughness` is permitted, `Roughness * 2.0` is permitted.

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    s: Roughness = 0.25
    a: f32 = r + s
    b: f32 = r * 2.0
    c: f32 = 2.0 * r
    print(a)
    print(b)
    print(c)
#$ stdout: 0.7511

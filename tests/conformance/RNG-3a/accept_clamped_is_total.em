#$ test: run-pass
#$ rules: RNG-3a
## "It is **total**: it has no failure mode and introduces no `Panic` and no
## `RuntimeCheck(k)`." Both endpoints, and a value between them.

type Roughness = f32 in 0.25 ..= 0.75

fn main():
    lo: f32 = Roughness.clamped(0.0)
    mid: f32 = Roughness.clamped(0.5)
    hi: f32 = Roughness.clamped(9.0)
    print(lo)
    print(mid)
    print(hi)
#$ stdout: 0.250.50.75
#$ assert-c: !contains("ember_panic")

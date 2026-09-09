#$ test: compile-fail
#$ rules: RNG-3
## IV.2a's worked diagnostic, verbatim.

type Roughness = f32 in 0.0 ..= 1.0

fn main():
    roughness: Roughness = 1.4    #$ error[E2211]: 1.4 is outside `Roughness`
    print(1)

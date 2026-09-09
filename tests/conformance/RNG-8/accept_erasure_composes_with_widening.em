#$ test: run-pass
#$ rules: RNG-8, TYP-5
## "Range erasure is a distinct coercion step that composes with the widening
## rules above, so `Roughness -> f32 -> f64` and `Percent -> u8 -> u32` are
## coercions."

type Roughness = f32 in 0.0 ..= 1.0
type Percent   = u8  in 0 ..= 100

fn wide(x: f64) -> f64: return x
fn big(x: u32) -> u32: return x

fn main():
    r: Roughness = 0.5
    p: Percent = 40
    print(wide(r))
    print(big(p))
#$ stdout: 0.540

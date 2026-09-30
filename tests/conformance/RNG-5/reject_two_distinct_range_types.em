#$ test: compile-fail
#$ rules: RNG-5
## "Arithmetic between two distinct nominal range types is rejected (`E2214`)
## unless at least one operand is explicitly converted to its representation."
## D-399 — the help named `x as f32`, which `as` refuses for a range type
## (`[TYP-6]` lists no range type); a range value becomes its representation
## at a coercion site (`[RNG-2]`).

type Roughness = f32 in 0.0 ..= 1.0
type Metallic  = f32 in 0.0 ..= 1.0

fn main():
    r: Roughness = 0.5
    m: Metallic = 0.5
    bad = r + m       #$ error[E2214]: `+` is not defined between `Roughness` and `Metallic`
    #$ help: convert one side to `f32` at a place of that type first, as `x: f32 = a` does
    print(1)

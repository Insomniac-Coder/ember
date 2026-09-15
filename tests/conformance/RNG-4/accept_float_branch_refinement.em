#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 1

type Unit = f32 in 0.0 ..= 1.0

fn as_unit(value: f32) -> Unit:
    # Strict comparisons are intentionally widened to closed interval facts.
    # The widened fact is conservative but is enough to prove this conversion.
    if value > 0.0 and value < 1.0:
        return value
    return Unit.clamped(0.0)

fn main():
    result = as_unit(0.5)
    if result == 0.5:
        println(1)
    else:
        println(0)

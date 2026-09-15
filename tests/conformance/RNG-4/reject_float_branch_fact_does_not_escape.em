#$ test: compile-fail
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping

type Unit = f32 in 0.0 ..= 1.0

fn not_proven(value: f32) -> Unit:
    if value > 0.0 and value < 1.0:
        pass
    return value  #$ error[E2215]: a `Unit` cannot be built from a value this is not known to be in range

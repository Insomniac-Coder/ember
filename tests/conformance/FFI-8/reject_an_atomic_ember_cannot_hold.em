#$ test: compile-fail
#$ rules: FFI-8
#$ warning[W5002]: `level_peek` is not imported from `atomic_levels.h`: parameter `level`: `_Atomic(float)`: `Atomic[T]` holds an integer, a `bool` or a pointer ([THR-14])
# `[FFI-8]` — an `_Atomic T` whose `T` `Atomic[T]` cannot hold (`[THR-14]`: an integer, a `bool`
# or a pointer), or that C lays out otherwise than `T`, is `E5016`: what uses it is not imported.

import c "atomic_levels.h"

fn main():
    level = c.level_peek    #$ error[E5016]

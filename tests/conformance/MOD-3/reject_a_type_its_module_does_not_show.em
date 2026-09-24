#$ test: compile-fail
#$ rules: MOD-3, MOD-2
# `[MOD-2]` — a type named through its module must be visible from here; one
# the module does not declare is not found there.

import support.geometry

fn hide(h: geometry.Hidden) -> int:   #$ error[E1052]: `Hidden` is private to `support.geometry`
    return 0

fn lose(n: geometry.Nowhere) -> int:   #$ error[E1010]: cannot find type `Nowhere` in module `geometry`
    return 0

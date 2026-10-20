#$ test: compile-fail
#$ rules: FFI-42
#$ warning[W5002]: `Vec2.operator!=` is not imported from `operators.hpp`
#$ warning[W5002]: `Vec2.operator<` is not imported from `operators.hpp`
# `[FFI-42]` — an operator implements its Ember operator interface only when its call is asserted
# safe (`[FFI-2]`: an interface's method is never `unsafe`): `operator-` with an operand, which
# the overlay names no entry for, is the `unsafe` method `sub`, so `a - b` has no `Sub`, and
# `a.sub(b)` needs `unsafe`. `operator<` maps to no interface (`Ord` is `cmp`): it is not imported.

import cpp "operators.hpp" with (overlay = "overlays/operators.em")

fn main():
    a = cpp.Vec2(x = 1, y = 2)
    b = cpp.Vec2(x = 10, y = 20)
    c = a - b    #$ error[E2020]: `-` cannot be applied
    d = a < b    #$ error[E2040]: does not implement `Ord`, which `<` needs
    e = a.sub(b)    #$ error[E3100]: calling an `unsafe fn` requires an `unsafe` block

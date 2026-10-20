#$ test: compile-fail
#$ rules: FFI-39
#$ warning[W5002]: an overload
#$ warning[W5002]: an overload
# `[FFI-39]` — `virtuals=[…]` names methods that are virtual in the header: naming one that is not
# is `E5057`, and the class gets no trampoline.

import cpp "layers.hpp" with (overlay = "overlays/not_virtual.em")    #$ error[E5057]: `virtuals` names `id`, which is not virtual in `Layer`

fn main():
    pass

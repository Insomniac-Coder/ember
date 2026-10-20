#$ test: compile-fail
#$ rules: FFI-43, FFI-24
#$ error[E5062]: the entry for `risky` declares it both `noexcept` and throwing
# `[FFI-43]` — an overlay entry that marks a function both `noexcept` and throwing is `E5062`: a C++
# function has one exception policy. (`@ffi(noexcept=false)` declares a function throwing; on one
# the header declares non-throwing, such as `stated`'s `throw()`, the header's word stands.)

import cpp "policies.hpp" with (overlay = "overlays/contradictory.em")

fn main():
    pass

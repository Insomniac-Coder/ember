#$ test: compile-fail
#$ rules: FFI-43
#$ error[E5062]: the overlays disagree on `risky`: an earlier one declares it `noexcept`, this one throwing
# `[FFI-43]` — two composed overlays that disagree on a function's exception policy are `E5062`, not
# the plain second entry of `E5010`: a C++ function has one policy whichever overlays compose (a
# later overlay replaces it only by writing `override`).

import cpp "policies.hpp" with (overlay = ["overlays/asserting.em", "overlays/declaring.em"])

fn main():
    pass

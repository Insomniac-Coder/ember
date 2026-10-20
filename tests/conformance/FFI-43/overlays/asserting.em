# One of the overlays ../reject_composed_overlays_that_disagree.em composes: `risky` asserted
# `noexcept`.

unsafe overlay cpp "policies.hpp":
    @ffi(noexcept)
    fn risky(x)

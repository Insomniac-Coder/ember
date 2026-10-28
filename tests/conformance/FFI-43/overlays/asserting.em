# One of the overlays ../reject_composed_overlays_that_disagree.em composes: `risky` asserted
# `noexcept`.

unsafe overlay cpp "policies.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(noexcept)
    fn risky(x)

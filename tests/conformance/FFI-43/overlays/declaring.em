# One of the overlays ../reject_composed_overlays_that_disagree.em composes: `risky` declared
# throwing.

unsafe overlay cpp "policies.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(noexcept=false)
    fn risky(x)

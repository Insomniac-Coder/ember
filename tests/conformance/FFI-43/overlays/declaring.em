# One of the overlays ../reject_composed_overlays_that_disagree.em composes: `risky` declared
# throwing.

unsafe overlay cpp "policies.hpp":
    @ffi(noexcept=false)
    fn risky(x)

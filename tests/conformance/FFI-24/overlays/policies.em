# The overlay of ../accept_an_overlay_asserts_noexcept.em and ../run_fail_an_asserted_noexcept_function_threw.em.

unsafe overlay cpp "policies.hpp":
    @ffi(noexcept)
    fn risky(x)

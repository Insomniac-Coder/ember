# The overlay of ../accept_a_declared_throwing_function_is_caught.em.

unsafe overlay cpp "policies.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(noexcept=false)
    fn stated(x)

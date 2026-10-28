# The overlay of ../reject_contradictory_exception_policies.em: `risky` both `noexcept` and throwing.

unsafe overlay cpp "policies.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(noexcept, noexcept=false)
    fn risky(x)
    @ffi(noexcept=false)
    fn stated(x)

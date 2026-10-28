# The overlay of ../reject_a_policy_neither_derived_nor_declared.em: `sub` declared non-throwing.

unsafe overlay cpp "freestanding.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(noexcept)
    fn sub(a, b)

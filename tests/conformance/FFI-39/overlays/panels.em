# The overlay of ../reject_a_cpp_base_with_no_default_constructor_needs_init.em.

unsafe overlay cpp "panels.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["width"])
    type Panel

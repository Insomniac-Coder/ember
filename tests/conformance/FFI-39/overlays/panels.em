# The overlay of ../reject_a_cpp_base_with_no_default_constructor_needs_init.em.

unsafe overlay cpp "panels.hpp":
    @ffi(trampoline, virtuals=["width"])
    type Panel

# The overlay of ../reject_virtuals_names_a_method_that_is_not_virtual.em: `id` is not virtual.

unsafe overlay cpp "layers.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["on_update", "id"])
    type Layer

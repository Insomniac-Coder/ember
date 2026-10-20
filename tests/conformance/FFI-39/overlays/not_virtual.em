# The overlay of ../reject_virtuals_names_a_method_that_is_not_virtual.em: `id` is not virtual.

unsafe overlay cpp "layers.hpp":
    @ffi(trampoline, virtuals=["on_update", "id"])
    type Layer

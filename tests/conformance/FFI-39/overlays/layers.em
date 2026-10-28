# The overlay of the FFI-39 tests: Ember classes may derive from `Layer`, overriding `on_update`
# and `name`.

unsafe overlay cpp "layers.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["on_update", "name"])
    type Layer

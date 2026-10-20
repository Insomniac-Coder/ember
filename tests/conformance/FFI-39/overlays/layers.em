# The overlay of the FFI-39 tests: Ember classes may derive from `Layer`, overriding `on_update`
# and `name`.

unsafe overlay cpp "layers.hpp":
    @ffi(trampoline, virtuals=["on_update", "name"])
    type Layer

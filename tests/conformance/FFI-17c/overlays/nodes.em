# The overlay of the FFI-17c test: Ember classes may derive from `Node`.

unsafe overlay cpp "nodes.hpp":
    @ffi(trampoline, virtuals=["value"])
    type Node

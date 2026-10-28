# The overlay of the FFI-17c test: Ember classes may derive from `Node`.

unsafe overlay cpp "nodes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["value"])
    type Node

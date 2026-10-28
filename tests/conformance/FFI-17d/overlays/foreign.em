# The overlay of the FFI-17d tests with `owner="foreign"`: C++'s `delete` destroys a `Node`, and
# `keep` takes one for C++ to own.

unsafe overlay cpp "nodes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["value"], owner="foreign")
    type Node
    fn keep(node: owned)

# The overlay of ../run_fail_a_foreign_delete_of_an_object_ember_owns.em: `owner="ember"`, the
# default.

unsafe overlay cpp "nodes.hpp":
    @ffi(trampoline, virtuals=["value"])
    type Node

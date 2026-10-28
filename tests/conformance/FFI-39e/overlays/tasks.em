# The overlay of the FFI-39e tests: Ember classes may derive from `Task`.

unsafe overlay cpp "tasks.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["act", "step"])
    type Task

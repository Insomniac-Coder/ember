# The overlay of the FFI-39e tests: Ember classes may derive from `Task`.

unsafe overlay cpp "tasks.hpp":
    @ffi(trampoline, virtuals=["act", "step"])
    type Task

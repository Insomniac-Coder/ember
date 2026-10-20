# The overlay of the FFI-39d tests: Ember classes may derive from `Widget`, and `fire` is safe to
# call.

unsafe overlay cpp "widgets.hpp":
    @ffi(trampoline, virtuals=["on_event"])
    type Widget
    fn fire(widget, code)

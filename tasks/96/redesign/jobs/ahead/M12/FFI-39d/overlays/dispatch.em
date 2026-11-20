# The overlay of the FFI-39d test: Ember classes may derive from `Widget`, whose objects C++'s
# `delete` ends once they are handed over; `hold` keeps the widget it is given, and `dispatch_all`
# has no `no_callbacks`.

unsafe overlay cpp "dispatch.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, owner="foreign")
    type Widget
    fn hold(widget: owns)
    fn dispatch_all()

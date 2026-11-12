# The overlay of the FFI-39c tests: Ember classes may derive from `Widget`, whose objects C++'s
# `delete` ends once they are handed over (`owner="foreign"`); `add_widget` keeps the widget it is
# given (`owns`), `draw_once` only borrows it for the call.

unsafe overlay cpp "widgets.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["draw"], owner="foreign")
    type Widget
    fn add_widget(widget: owns)
    fn draw_added()
    fn remove_widget()
    fn draw_once(widget)

# The overlay of the FFI-39c tests: Ember classes may derive from `Widget`; `add_widget` keeps the
# widget it is given (`retained`), `draw_once` only borrows it for the call.

unsafe overlay cpp "widgets.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["draw"])
    type Widget
    fn add_widget(widget: retained)
    fn draw_added()
    fn remove_widget()
    fn draw_once(widget)

# The overlay of the FFI-2 tests: the facts `widgets.h` cannot state ([FFI-2], [FFI-11]).

unsafe overlay c "widgets.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(destroys(widget_free))
    type Widget

    fn widget_new(id) -> returns_owned(destructor=widget_free) one
    fn widget_take(widget: owned one)
    fn widget_sum(items: borrowed count(n), n)
    fn widget_fill(items: borrowed count(n), n, value)
    fn widget_label(id) -> borrowed nul_terminated from(static)
    fn widget_length(text: borrowed nul_terminated)
    @ffi(effects=[])
    fn widget_add(a, b)
    fn widget_double(value: borrowed one)
    fn widget_freed()
    rename widget_add as add
    hide widget_free

    extend Size:
        pub fn area(self) -> i32:
            return self.w * self.h

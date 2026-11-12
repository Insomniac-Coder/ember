#$ test: compile-fail
#$ rules: FFI-36a
# `[FFI-36a]` — a foreign type has one destructor: every
# `returns_owned(destructor=f)` giving an `owns T` and `T`'s
# `@ffi(destroys(f))` name the same `f`, so an `owns T` of a foreign type stays
# one word. A result naming another is `E5002`, naming both lines; so is a
# second result naming another where the declaration names none.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    type Gadget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_destroy(widget: *mut Widget)
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget)
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_drop(gadget: *mut Gadget)
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget
    @ffi(result(returns_owned(destructor=widget_destroy), one), link_name="widget_new")    #$ error[E5002]: `Widget` is ended by `widget_free` (line 10), and line 23 says `widget_destroy`; a foreign type has one destructor
    safe fn widget_other(id: i32) -> owns Widget
    @ffi(result(returns_owned(destructor=gadget_close), one))
    safe fn gadget_new(id: i32) -> owns Gadget
    @ffi(result(returns_owned(destructor=gadget_drop), one), link_name="gadget_new")    #$ error[E5002]: `Gadget` is ended by `gadget_close` (line 25), and line 27 says `gadget_drop`; a foreign type has one destructor
    safe fn gadget_other(id: i32) -> owns Gadget

fn main():
    pass

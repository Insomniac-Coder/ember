#$ test: parse-pass
#$ rules: FFI-36a, FFI-11, FFI-33b
# `[FFI-36a]`, `[FFI-11]` — where the contract records the transfer, no `adopt`
# is written: a result contracted `returns_owned(destructor=f)` is an `owns T`
# of the foreign type, which the caller owns (`Option` of one with `nullable`,
# `None` for null), and a parameter contracted `owns` takes one handed over,
# giving C its object and freeing nothing. An `owns T` reads as its `T`: a
# parameter contracted `borrowed one` views the object for the call. A
# status-returning destructor and a `threads(creator)` type work as with
# `adopt`. The C side is `ember_driver/tests/fixtures/foreign_objects.c`; the
# milestone `adopted_foreign_objects_are_freed_by_their_destructor` links and
# runs it.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @ffi(threads(creator))
    type Gadget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @ffi(param(widget, borrowed, one))
    safe fn widget_id(widget: ref Widget) -> i32
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget) -> i32
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget
    @ffi(result(returns_owned(destructor=widget_free), one, nullable), link_name="widget_maybe")
    safe fn widget_or_none(id: i32) -> Option[owns Widget]
    @ffi(result(returns_owned(destructor=gadget_close), one))
    safe fn gadget_new(id: i32) -> owns Gadget
    @ffi(param(widget, owns, one))
    safe fn widget_take(owns widget: owns Widget)
    @ffi(param(widget, owns, one, nullable), link_name="widget_take")
    safe fn widget_take_maybe(owns widget: Option[owns Widget])
    @ffi(effects=[])
    safe fn objects_live() -> i32
    @ffi(effects=[])
    safe fn objects_freed() -> i32

fn id_of(widget: owns Widget) -> i32:
    return widget_id(widget)

fn main():
    first = widget_new(2)
    println(id_of(first), objects_live())
    match widget_or_none(-1):
        Some(_) => println("unexpected")
        None => println("none")
    match widget_or_none(3):
        Some(third) => println(id_of(third), objects_live())
        None => println("unexpected")
    println(objects_freed(), objects_live())
    widget_take(owns first)
    println(objects_freed(), objects_live())
    widget_take_maybe(owns widget_or_none(4))
    widget_take_maybe(owns None)
    println(objects_freed(), objects_live())
    _gadget = gadget_new(5)
    println(objects_live())

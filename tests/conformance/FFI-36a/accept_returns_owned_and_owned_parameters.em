#$ test: parse-pass
#$ rules: FFI-36a, FFI-11, FFI-33b
# `[FFI-36a]`, `[FFI-11]` — where the contract records the transfer, no `adopt`
# is written: a result contracted `returns_owned(destructor=f)` is a
# `ForeignBox[T]` the caller owns (`Option` of one with `nullable`, `None` for
# null), and a parameter contracted `owned` takes the box, giving C its object
# and freeing nothing. A status-returning destructor and a `threads(creator)`
# type work as with `adopt`. The C side is
# `ember_driver/tests/fixtures/foreign_objects.c`; the milestone
# `adopted_foreign_objects_are_freed_by_their_destructor` links and runs it.

from std.ffi import ForeignBox

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    @ffi(threads(creator))
    type Gadget
    fn widget_free(widget: *mut Widget)
    fn widget_id(widget: *mut Widget) -> i32
    fn gadget_close(gadget: *mut Gadget) -> i32
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> ForeignBox[Widget]
    @ffi(result(returns_owned(destructor=widget_free), one, nullable), link_name="widget_maybe")
    safe fn widget_or_none(id: i32) -> Option[ForeignBox[Widget]]
    @ffi(result(returns_owned(destructor=gadget_close), one))
    safe fn gadget_new(id: i32) -> ForeignBox[Gadget]
    @ffi(param(widget, owned, one))
    safe fn widget_take(owned widget: ForeignBox[Widget])
    @ffi(param(widget, owned, one, nullable), link_name="widget_take")
    safe fn widget_take_maybe(owned widget: Option[ForeignBox[Widget]])
    @ffi(effects=[])
    safe fn objects_live() -> i32
    @ffi(effects=[])
    safe fn objects_freed() -> i32

fn id_of(widget: ForeignBox[Widget]) -> i32:
    id: i32 = 0
    unsafe:  # SAFETY(ffi): the box keeps the widget alive for the call
        id = widget_id(widget.as_ptr())
    return id

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
    widget_take(first)
    println(objects_freed(), objects_live())
    widget_take_maybe(widget_or_none(4))
    widget_take_maybe(None)
    println(objects_freed(), objects_live())
    _gadget = gadget_new(5)
    println(objects_live())

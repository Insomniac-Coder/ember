#$ test: parse-pass
#$ rules: FFI-36a, FFI-36b, FFI-33b
# `[FFI-36a]` — `adopt[T](h)` gives an `owns T` of the foreign object at `h`;
# its end frees it, once, with the destructor the declaration of `T` names
# (`@ffi(destroys(f))`). A destructor with a status is called and its status
# dropped. Each object's address leaves the runtime's record when its owner
# ends (`[FFI-36b]`), and a `threads(creator)` one ends on its own thread
# (`[FFI-33b]`). An `owns T` reads as its `T`: a parameter contracted
# `borrowed one` views the object. The C side is
# `ember_driver/tests/fixtures/foreign_objects.c`; the milestone
# `adopted_foreign_objects_are_freed_by_their_destructor` links and runs it.

from std.ffi import adopt
from std.mem import drop

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free), handle)
    type Widget
    @ffi(destroys(gadget_close), threads(creator))
    type Gadget
    @safety("C gives a fresh widget, which the caller owns")
    unsafe fn widget_new(id: i32) -> *mut Widget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @ffi(param(widget, borrowed, one))
    safe fn widget_id(widget: ref Widget) -> i32
    @safety("C gives a fresh gadget, which the caller owns")
    unsafe fn gadget_new(id: i32) -> *mut Gadget
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget) -> i32
    @ffi(effects=[])
    safe fn objects_live() -> i32
    @ffi(effects=[])
    safe fn objects_freed() -> i32

fn make(id: i32) -> owns Widget:
    # SAFETY: `widget_new` gives a fresh widget nothing else owns
    unsafe:
        return adopt(widget_new(id))

fn read(widget: owns Widget) -> i32:
    return widget_id(widget)

fn make_gadget(id: i32) -> owns Gadget:
    # SAFETY: a fresh gadget, owned only by this `owns Gadget`
    unsafe:
        return adopt[Gadget](gadget_new(id))

fn main():
    first = make(3)
    println(read(first), objects_live())
    gadget = make_gadget(5)
    if true:
        second = make(4)
        println(read(second), objects_live())
    println(objects_freed(), objects_live())
    held = [first]
    println(read(held[0]))
    held.clear()
    println(objects_freed(), objects_live())
    drop(gadget)
    println(objects_freed(), objects_live())

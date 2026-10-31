#$ test: parse-pass
#$ rules: FFI-36a, FFI-36b, FFI-33b
# `[FFI-36a]` — `adopt[T](h)` makes a `ForeignBox[T]` own the foreign object at
# `h`; the box frees it, once, with the destructor the declaration of `T`
# names (`@ffi(destroys(f))`), at the end of the box's life. A destructor with
# a status is called and its status dropped. Each box's address leaves the
# runtime's record when it ends (`[FFI-36b]`), and a `threads(creator)` box
# ends on its own thread (`[FFI-33b]`). The C side is
# `ember_driver/tests/fixtures/foreign_objects.c`; the milestone
# `adopted_foreign_objects_are_freed_by_their_destructor` links and runs it.

from std.ffi import adopt, ForeignBox
from std.mem import drop

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free), handle)
    type Widget
    @ffi(destroys(gadget_close), threads(creator))
    type Gadget
    unsafe fn widget_new(id: i32) -> *mut Widget
    unsafe fn widget_free(widget: *mut Widget)
    unsafe fn widget_id(widget: *mut Widget) -> i32
    unsafe fn gadget_new(id: i32) -> *mut Gadget
    unsafe fn gadget_close(gadget: *mut Gadget) -> i32
    @ffi(effects=[])
    safe fn objects_live() -> i32
    @ffi(effects=[])
    safe fn objects_freed() -> i32

fn make(id: i32) -> ForeignBox[Widget]:
    unsafe:  # SAFETY(ffi): `widget_new` gives a fresh widget nothing else owns
        return adopt(widget_new(id))

fn read(widget: ForeignBox[Widget]) -> i32:
    unsafe:  # SAFETY(ffi): the box keeps the widget alive for the call
        return widget_id(widget.as_ptr())

fn make_gadget(id: i32) -> ForeignBox[Gadget]:
    unsafe:  # SAFETY(ffi): a fresh gadget, owned only by this box
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

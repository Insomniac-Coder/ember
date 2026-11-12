#$ test: run-pass
#$ rules: FFI-36a, FFI-33b, FFI-17a
#$ profiles: debug, development, release
#$ stdout: 8 8 true
#$ stdout: 16 16
# `[FFI-36a]` — a foreign type has one destructor, so an `owns T` of one holds nothing but the
# object's address: one word, as C's `T*` and C++'s `std::unique_ptr<T>` are (`[FFI-17a]`: "an
# `owns T` holds one address"), and an `Option` of one adds its tag. `[FFI-33b]` — two
# words for a type bound to the thread that made it: the address and that thread. (The
# destructors are the program's own C functions, so the test links on its own.)

from std.mem import size_of

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @ffi(destroys(context_free), threads(creator))
    type Context
    @ffi(link_name="sized_widget_free")
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @ffi(link_name="sized_context_free")
    @safety("`context` is a context C made, not freed yet")
    unsafe fn context_free(context: *mut Context)

pub extern "C" fn sized_widget_free(widget: *mut Widget):
    pass

pub extern "C" fn sized_context_free(context: *mut Context):
    pass

fn main():
    println(size_of[owns Widget](), size_of[*mut Widget](), size_of[Option[owns Widget]]() <= 16)
    println(size_of[owns Context](), size_of[*mut Context]() * 2)

#$ test: compile-fail
#$ rules: FFI-36a, FFI-36, TYP-45
# `[FFI-36a]` — a foreign object Ember owns is an `owns T` of its type: 0.9.10's `ForeignBox[T]`
# is gone, as `Box` is (`[TYP-45]`). Naming it is `E1010`, whose message gives what to write
# (`owns Widget`), and so is importing it, which an `owns T` needs no import for.

from std.ffi import adopt, ForeignBox    #$ error[E1010]: `ForeignBox` is not a type of Ember: write `owns T`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @safety("C gives a fresh widget, which the caller owns")
    unsafe fn widget_new(id: i32) -> *mut Widget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)

fn make(id: i32) -> ForeignBox[Widget]:    #$ error[E1010]: `ForeignBox` is not a type of Ember: write `owns Widget`
    # SAFETY: `widget_new` gives a fresh widget nothing else owns
    unsafe:
        return adopt(widget_new(id))

fn main():
    _widget = make(1)

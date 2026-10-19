#$ test: parse-pass
#$ rules: FFI-36b, FFI-36a
# `[FFI-36b]` — adopting one live foreign object twice is caught at run time in every profile: the
# runtime records each address a `ForeignBox` owns with where it was taken, and the second `adopt`
# panics naming both places, before a second owner could free it twice. Here the two calls adopt
# two locals holding one pointer, which the compile-time check (`E5068`, one local) does not follow.
# The milestone `adopted_foreign_objects_are_freed_by_their_destructor` builds this program with
# `ember_driver/tests/fixtures/foreign_objects.c` and runs it in the debug and development profiles.

from std.ffi import adopt

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    fn widget_new(id: i32) -> *mut Widget
    fn widget_free(widget: *mut Widget)

fn main():
    unsafe:  # SAFETY(ffi): the test adopts one widget twice
        widget = widget_new(1)
        again = widget
        first = adopt(widget)
        second = adopt(again)
    println("unreachable")

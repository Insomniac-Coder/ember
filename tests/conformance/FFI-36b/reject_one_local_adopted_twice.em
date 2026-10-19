#$ test: compile-fail
#$ rules: FFI-36b
#$ profiles: debug
# `[FFI-36b]` — two `adopt` calls on one local in one function would make two owners of one live
# object, each freeing it: `E5068` at compile time. Writing the local between the calls makes it
# another object, and two calls in branches that exclude each other never both run.

from std.ffi import adopt

unsafe extern "C":
    @ffi(destroys(widget_free))
    type Widget
    fn widget_new(id: i32) -> *mut Widget
    fn widget_free(widget: *mut Widget)

fn twice():
    unsafe:  # SAFETY(ffi): the test adopts one widget twice
        widget = widget_new(1)
        _first = adopt(widget)
        _second = adopt(widget)  #$ error[E5068]: already adopted

fn renewed():
    unsafe:  # SAFETY(ffi): each `adopt` takes a new widget
        widget = widget_new(1)
        _first = adopt(widget)
        widget = widget_new(2)
        _second = adopt(widget)

fn either(left: bool):
    unsafe:  # SAFETY(ffi): only one branch runs
        widget = widget_new(1)
        if left:
            _first = adopt(widget)
        else:
            _second = adopt(widget)

fn after_a_branch(left: bool):
    unsafe:  # SAFETY(ffi): the test adopts one widget twice when `left`
        widget = widget_new(1)
        if left:
            _first = adopt(widget)
        _second = adopt(widget)  #$ error[E5068]: already adopted

fn main():
    twice()
    renewed()
    either(true)
    after_a_branch(false)

#$ test: compile-fail
#$ rules: OWN-6, OWN-2, OWN-12
# `[OWN-6]` — there is no `mem.forget`: a value always ends with its clean-up. A call is `E1010`,
# whose message says how C keeps a value instead: handed to a parameter its declaration says it
# takes, or, inside `unsafe:`, given with `into_raw()` (`[OWN-12]`).

import std.mem

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(widget_free))
    type Widget
    @safety("`widget` is a widget C made, not freed yet")
    unsafe fn widget_free(widget: *mut Widget)
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn widget_new(id: i32) -> owns Widget

struct Resource:
    pub value: i32

fn main():
    first = widget_new(1)
    mem.forget(first)    #$ error[E1010]: `mem.forget` is gone: a value always ends with its clean-up; to give the widget to a C function that keeps it, hand it to a parameter its declaration says it takes (`widget_take(owns first)`), or, inside `unsafe:`, give it `first.into_raw()`
    mem.forget(Resource(98))    #$ error[E1010]: `mem.forget` is gone: a value always ends with its clean-up; to give the resource to a C function that keeps it, hand it to a parameter its declaration says it takes (`resource_take(owns Resource(98))`), or, inside `unsafe:`, give it `Resource(98).into_raw()`

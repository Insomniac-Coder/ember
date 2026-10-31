#$ test: compile-fail
#$ rules: FFI-36a, FFI-11
# `[FFI-11]` — a `ForeignBox` crosses a `safe fn` only as the object its
# contract moves: `owned` takes an `owned` box, `returns_owned(destructor=f)`
# gives one, freed by a foreign `f` taking one `*mut T`; nothing else
# carries one.

from std.ffi import ForeignBox

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Widget
    @safety("callers pass what the C function `widget_free` requires")
    unsafe fn widget_free(widget: *mut Widget)
    @safety("callers pass what the C function `widget_count` requires")
    unsafe fn widget_count(n: i32) -> i32
    @ffi(param(widget, owned, one))
    safe fn lends(widget: ForeignBox[Widget])    #$ error[E5002]: `owned` contract for `widget` needs `owned widget: ForeignBox[T]`
    safe fn hidden(owned widget: ForeignBox[Widget])    #$ error[E5002]: a `ForeignBox` parameter crosses only as an object C takes
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn plain() -> i32    #$ error[E5002]: `returns_owned` needs a `ForeignBox[T]` result
    @ffi(result(returns_owned(destructor=widget_count), one))    #$ error[E5002]: `widget_count` frees a `Widget`
    safe fn counted() -> ForeignBox[Widget]
    safe fn unannounced() -> ForeignBox[Widget]    #$ error[E5002]: a `ForeignBox` result needs `result(returns_owned(destructor=f), one)`

fn main():
    pass

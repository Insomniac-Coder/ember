#$ test: compile-fail
#$ rules: FFI-36a, FFI-11
# `[FFI-11]` — an `owns T` of a foreign type crosses a `safe fn` only as the
# object its contract moves: `owns` takes one handed over,
# `returns_owned(destructor=f)` gives one, ended by a foreign `f` taking one
# `*mut T`; nothing else carries one.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Widget
    @safety("callers pass what the C function `widget_free` requires")
    unsafe fn widget_free(widget: *mut Widget)
    @safety("callers pass what the C function `widget_count` requires")
    unsafe fn widget_count(n: i32) -> i32
    @ffi(param(widget, owns, one))
    safe fn lends(widget: owns Widget)    #$ error[E5002]: `owns` contract for `widget` needs `owns widget: owns T`, a foreign type's
    safe fn hidden(owns widget: owns Widget)    #$ error[E5002]: an `owns` value of a foreign type crosses only as an object C takes
    @ffi(result(returns_owned(destructor=widget_free), one))
    safe fn plain() -> i32    #$ error[E5002]: `returns_owned` needs an `owns T` result of a foreign type
    @ffi(result(returns_owned(destructor=widget_count), one))    #$ error[E5002]: `widget_count` frees a `Widget`
    safe fn counted() -> owns Widget
    safe fn unannounced() -> owns Widget    #$ error[E5002]: an `owns` result of a foreign type needs `result(returns_owned(destructor=f), one)`

fn main():
    pass

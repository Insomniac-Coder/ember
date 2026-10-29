#$ test: compile-fail
#$ rules: FFI-36a, FFI-11
# `[FFI-36a]` — `adopt[T]` reads the destructor from the declaration of `T`:
# a foreign type whose contract names none, a type that is not foreign, a
# destructor that does not take a `*mut T`, and a call outside `unsafe` (it is
# an `unsafe fn`) are each refused; so is a contract word a foreign type does
# not take.

from std.ffi import adopt

struct Plain:
    x: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Bare
    @ffi(destroys(bare_count))    #$ error[E5002]: `bare_count` frees a `Counted`, so it takes one `*mut Counted`
    type Counted
    @ffi(destroys(widget_free))
    type Widget
    @ffi(destroys(gone), shared)    #$ error[E0104]: this is not a foreign type's contract word
    type Odd
    @safety("callers pass what the C function `bare_new` requires")
    fn bare_new() -> *mut Bare
    @safety("callers pass what the C function `counted_new` requires")
    fn counted_new() -> *mut Counted
    @safety("callers pass what the C function `bare_count` requires")
    fn bare_count(n: i32) -> i32
    @safety("callers pass what the C function `widget_free` requires")
    fn widget_free(widget: *mut Widget)

fn main():
    unsafe:  # SAFETY(ffi): each pointer is fresh
        a = adopt(bare_new())    #$ error[E5002]: the declaration of `Bare` names no destructor
        b = adopt(null[*mut Plain]())    #$ error[E5002]: `Plain` is not a foreign type
        c = adopt(counted_new())
    d = adopt(null[*mut Widget]())    #$ error[E3100]: `adopt(null[*mut Widget]())` (line 36) calls an `unsafe fn`, whose callers must make true what its `@safety` says

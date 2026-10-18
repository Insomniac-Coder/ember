## `std.ffi` — owned C strings for Part XVI. A `CString` owns UTF-8 bytes
## followed by exactly one NUL terminator. Its `as_cstr()` view borrows this
## owner and is supplied by the compiler until raw-pointer view construction
## can express the invariant in the library.

import std.core

## `[FFI-15]` (ODR-081) — report the first interior NUL byte, rather than
## truncating a C argument or panicking on runtime input.
pub struct NulError:
    pub index: usize

pub struct CString:
    bytes: String

extend str:
    pub fn to_cstring(self) -> Result[CString, NulError]:
        match self.find("\0"):
            Some(index):
                return Err(NulError(index as usize))
            None:
                bytes = self.to_string()
                bytes.push('\0')
                return Ok(CString(bytes))

extend String:
    pub fn to_cstring(self) -> Result[CString, NulError]:
        return self.as_str().to_cstring()

## `[FFI-36a]` — `ForeignBox[T]` owns a foreign object: its address, and the foreign function
## that frees it, which the box's end calls once. A box is made by `adopt[T](h)` (the destructor
## is the one the declaration of `T` names, `@ffi(destroys(f))`) or by a foreign function whose
## result is contracted `returns_owned(destructor=f)`, and given back to C by a parameter
## contracted `owned`, which frees nothing. `[FFI-36b]` — the runtime records every address a box
## owns, with where it was taken, so taking a live one again panics naming both places, in every
## profile (`foreign_release` takes it out of the record). `[FFI-33b]` — a box of a type whose
## contract says `threads(creator)` records the thread that made it and, in the debug profile,
## panics when it ends on another; that is what lets a box move between threads at all (it is
## `Send`): the thread rule is the contract's, checked where it can be.
pub struct ForeignBox[T]:
    pointer: *mut T
    destructor: extern "C" fn(*mut T)
    creator: u32

    ## The object's address, for a call that borrows it.
    pub fn as_ptr(self) -> *mut T:
        return self.pointer

    fn drop(mut self):
        foreign_release(self.pointer, self.creator)
        destroy = self.destructor
        destroy(self.pointer)

unsafe extend[T] ForeignBox[T] implements Send:  # SAFETY: a box's thread rule is its contract's: `threads(creator)` is checked at its end ([FFI-33b])
    pass

## `[FFI-36a]` — take ownership of the foreign object at `h`: its box frees it with the
## destructor the declaration of `T` names (`@ffi(destroys(f))`). Compiler-known: the destructor
## is read from that declaration, and one that says `@ffi(adopt=false)` is `E5052`.
@safety("`h` is a live foreign object of type `T` that nothing else owns, and `T`'s destructor frees it.")
pub unsafe fn adopt[T](h: *mut T) -> ForeignBox[T]:  # SAFETY: the declaration states the obligation; the compiler supplies the body
    pass

## `[FFI-36b]`, `[FFI-33b]` — a box's end: the address leaves the runtime's record, and a box
## that records its creator is checked to end on that thread (debug). Compiler-known.
fn foreign_release[T](pointer: *mut T, creator: u32):
    pass

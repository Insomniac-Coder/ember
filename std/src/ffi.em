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

## `[FFI-8]` — C's types by their C names, as `import c` writes them (`int` is `c_int`). Those
## whose type differs between targets are written here as one target's; the compiler gives each
## the type of the target it builds for: `c_char` is `u8` where C's `char` is unsigned (ARM and
## other non-x86 Linux targets), `c_long` and `c_ulong` are 32 bits on Windows and on 32-bit
## targets, and `c_wchar` is `u16` on Windows (not portable).
pub type c_char = i8
pub type c_schar = i8
pub type c_uchar = u8
pub type c_short = i16
pub type c_ushort = u16
pub type c_int = i32
pub type c_uint = u32
pub type c_long = i64
pub type c_ulong = u64
pub type c_longlong = i64
pub type c_ulonglong = u64
pub type c_float = f32
pub type c_double = f64
pub type c_wchar = i32

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

## `[FFI-23]` — `Retained[T]` is the only way to let foreign code keep Ember memory.
## `Retained.pin(v)` takes `v`, a class handle or a `Box`: for a class handle the token holds a
## strong count of the object (it is retained) and the object's header gets its pinned flag, which
## stays (`[HR-20]`: an object foreign code may hold is never moved); for a `Box` the token owns it.
## `tok.ptr()` is the object's address, which C keeps, typed by what the token holds (ODR-132): an
## `ObjectPtr[C]` for a class `C`, a `*mut U` for a `Box[U]`, a `*mut u8` for a `Box[dyn I]`, whose
## value's type is not known. Compiler-known. Dropping the token releases the object (the class
## handle's count, or the box and its value). A parameter contracted `retained` takes a token and
## gives C its address (`[FFI-11]`).
pub struct Retained[T]:
    value: T

    pub fn pin(owned value: T) -> Retained[T]:
        retained_pin(value)
        return Retained(value)

## `[FFI-23]` (ODR-132) — the address of a class object, for foreign code to keep or to be handed
## back: no count of its own, so it says nothing about how long the object lives (a `Retained`
## token keeps it alive). To C it is one pointer to the object. Two are equal when they are the
## address of one object.
@derive(Copy)
pub struct ObjectPtr[C]:
    address: *mut u8

## `[FFI-23]` — compiler-known: a class object's pinned flag is set, a `Box` needs nothing, and any
## other value is refused.
fn retained_pin[T](value: T):
    pass


## `[FFI-36b]`, `[FFI-33b]` — a box's end: the address leaves the runtime's record, and a box
## that records its creator is checked to end on that thread (debug). Compiler-known.
fn foreign_release[T](pointer: *mut T, creator: u32):
    pass

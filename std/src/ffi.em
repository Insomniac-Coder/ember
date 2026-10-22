## `std.ffi` — owned C strings for Part XVI. A `CString` owns UTF-8 bytes
## followed by exactly one NUL terminator. Its `as_cstr()` view borrows this
## owner and is supplied by the compiler until raw-pointer view construction
## can express the invariant in the library.

import std.core
from std.fmt import Formatter, FmtError
import std.mem
from std.string import Utf8Error

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

## `[TXT-7]` — text is UTF-8 everywhere; `WideString` is UTF-16 for the APIs that need it
## (Windows' `W` functions), converted explicitly: `s.to_wide()` encodes a `str` (an interior NUL
## is a `NulError` at its byte, as for `to_cstring`) and adds one 0 unit; `w.as_ptr()` is the
## terminated units for the call; `WideString.from_units(units)` takes C's units up to the first 0
## and `w.to_string()` decodes them, an unpaired surrogate becoming U+FFFD.
pub struct WideString:
    units: Array[u16]

    ## The units, without the terminating 0.
    pub fn units(self) -> Span[u16]:
        return self.units[0..self.units.len() - 1]

    ## The first unit's address: the units and their terminating 0, for a foreign call.
    pub fn as_ptr(self) -> *u16:
        return self.units.as_span().as_ptr()

    pub fn from_units(units: Span[u16]) -> WideString:
        kept: Array[u16] = []
        for unit in units:
            if unit == 0:
                break
            kept.push(unit)
        kept.push(0)
        return WideString(kept)

    pub fn to_string(self) -> String:
        out = String()
        units = self.units()
        i = 0
        while i < units.len():
            unit = units[i] as u32
            code = unit
            i += 1
            if unit >= 0xD800 and unit < 0xDC00 and i < units.len():
                low = units[i] as u32
                if low >= 0xDC00 and low < 0xE000:
                    code = 0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00)
                    i += 1
            match char.from_u32(code):
                Some(c):
                    out.push(c)
                None:
                    out.push('\u{FFFD}')
        return out

extend str:
    pub fn to_wide(self) -> Result[WideString, NulError]:
        units: Array[u16] = []
        for (at, c) in self.char_indices():
            code = c as u32
            if code == 0:
                return Err(NulError(at as usize))
            if code < 0x10000:
                units.push(code as u16)
            else:
                rest = code - 0x10000
                units.push((0xD800 + (rest >> 10)) as u16)
                units.push((0xDC00 + (rest & 0x3FF)) as u16)
        units.push(0)
        return Ok(WideString(units))

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

## `[HR-39]` — the failure a foreign call returns as a value: a C++ exception its thunk caught
## (`CppError`), or a C function's status other than its success (`[FFI-16]`'s error types, which
## the compiler makes implement this). `?` turns one into `std.hot.ReloadError.Foreign` inside a
## `migrate_from`.
pub interface ForeignFailure: Error:
    pass

## `[FFI-24]` — a C++ exception, caught by the thunk of the C++ function that threw it: the name
## of its type, and what its `what()` said (empty for an exception that is no `std::exception`).
## Its text is `<type>: <what>`.
@derive(Debug)
pub struct CppError:
    pub type_name: String
    pub what: String

extend CppError implements Display:
    ## It allocates: it writes into the formatter's text (`[STD-1]`).
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]:
        f.write_str(self.type_name)?
        f.write_str(": ")?
        return f.write_str(self.what)

extend CppError implements Error:
    pass

extend CppError implements ForeignFailure:
    pass

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

    ## The object, given up: the box frees nothing, and its address leaves the runtime's record
    ## (`[FFI-36b]`), so foreign code may own it — what a parameter contracted `owned` does
    ## (`[FFI-11]`), and how a `std::unique_ptr` parameter takes its object (`[FFI-17a]`).
    pub fn into_raw(owned self) -> *mut T:
        pointer = self.pointer
        foreign_release(self.pointer, self.creator)
        mem.forget(self)
        return pointer

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
## `[HR-20]` (ODR-194) — `on_relocate`, when given, lets a reload move the class object the token
## holds: the runtime calls it with the old and the new address right after commit publishes the
## move and before reclaim, so foreign code can follow it (a `Box`'s value never moves). Without
## one, the object is pinned and a reload that would move it is refused.
pub struct Retained[T]:
    value: T
    ## The runtime's record of `on_relocate` (`0`: none), which the token's end takes out.
    relocation: u64

    pub fn pin(owned value: T, on_relocate: Option[extern "C" fn(ObjectPtr[T], ObjectPtr[T])] = None) -> Retained[T]:
        relocation = retained_pin(value, on_relocate)
        return Retained(value, relocation)

    fn drop(mut self):
        retained_unpin(self.relocation)

## `[FFI-23]` (ODR-132) — the address of a class object, for foreign code to keep or to be handed
## back: no count of its own, so it says nothing about how long the object lives (a `Retained`
## token keeps it alive). To C it is one pointer to the object. Two are equal when they are the
## address of one object.
@derive(Copy)
pub struct ObjectPtr[C]:
    address: *mut u8

extend[C] ObjectPtr[C] implements Eq:
    ## Equal when they are the address of one object.
    fn eq(self, other: ObjectPtr[C]) -> bool:
        return self.address == other.address

## `[FFI-23]` — compiler-known: a class object's pinned flag is set, or with `on_relocate` its
## record made (`[HR-20]`), a `Box` needs nothing, and any other value is refused. The result is
## the record's number, `0` for none.
fn retained_pin[T](value: T, on_relocate: Option[extern "C" fn(ObjectPtr[T], ObjectPtr[T])]) -> u64:
    return 0

## `[HR-20]` — compiler-known: a token's `on_relocate` record goes.
fn retained_unpin(relocation: u64):
    pass


## `[FFI-36b]`, `[FFI-33b]` — a box's end: the address leaves the runtime's record, and a box
## that records its creator is checked to end on that thread (debug). Compiler-known.
fn foreign_release[T](pointer: *mut T, creator: u32):
    pass


## `[FFI-17a]` — the C++ object a bridge value owns (a `std::vector`, a `std::string`, a
## `std::shared_ptr`), as a declaration `import cpp` writes takes it: only ever behind a reference,
## so no Ember code reads or writes it.
unsafe extern "C":
    pub type CppObject

## `[FFI-17a]`, `[FFI-17e]` — a C++ `std::vector<T>` Ember owns, its elements in place. The import
## that made it gives the thunks of its element type, which each operation calls: `len`,
## `push_back` and its end are calls into C++, never inlined (`[FFI-17f]`). `span()` and
## `span_mut()` view the elements without copying and borrow the vector, so neither outlives it;
## `to_array()` copies them. Not `Copy`, `Send` or `Sync`: C++ makes a container no such promise.
pub struct CppVector[T]:
    object: *mut u8
    data: unsafe extern "C" fn(*mut u8) -> *mut T
    size: unsafe extern "C" fn(*mut u8) -> usize
    push: unsafe extern "C" fn(*mut u8, *T)
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the vector at `object`, with its element type's thunks: what `import cpp`
    ## makes of a `std::vector<T>`.
    @safety("`object` is a live `std::vector<T>` nothing else owns, and the functions are the thunks of its element type")
    pub unsafe fn from_thunks(object: *mut u8, data: unsafe extern "C" fn(*mut u8) -> *mut T, size: unsafe extern "C" fn(*mut u8) -> usize, push: unsafe extern "C" fn(*mut u8, *T), destroy: unsafe extern "C" fn(*mut u8)) -> CppVector[T]:  # SAFETY: the caller keeps the obligation
        return CppVector(object, data, size, push, destroy)

    ## The `std::vector` itself, for a call into C++.
    pub fn as_ptr(self) -> *mut u8:
        return self.object

    pub fn len(self) -> usize:
        size = self.size
        unsafe:  # SAFETY(ffi): the vector's own thunk, on the vector it came with
            return size(self.object)

    pub fn span(self) -> Span[T]:
        data = self.data
        unsafe:  # SAFETY(ffi): the vector's elements, which the view borrows the vector for
            return cpp_span(ref self, data(self.object), self.len())

    pub fn span_mut(mut self) -> MutSpan[T]:
        data = self.data
        count = self.len()
        unsafe:  # SAFETY(ffi): the vector's elements, viewed exclusively while the vector is
            return cpp_span_mut(ref mut self, data(self.object), count)

    pub fn push_back(mut self, value: T):
        push = self.push
        unsafe:  # SAFETY(ffi): the thunk copies the value into the vector
            push(self.object, ref_to_ptr(ref value))

    fn drop(mut self):
        destroy = self.destroy
        unsafe:  # SAFETY(ffi): the vector's end, once
            destroy(self.object)

extend[T: Copy] CppVector[T]:
    ## The elements, copied into an `Array`.
    pub fn to_array(self) -> Array[T]:
        out: Array[T] = []
        for item in self.span():
            out.push(item)
        return out

## `[FFI-17a]`, `[FFI-17e]` — a C++ `std::string` Ember owns: its bytes in place. `bytes()` views
## them and `to_str()` views them as text when they are UTF-8 (`[TXT-2]`), each borrowing the
## string; `String.from(c)` copies them, each invalid sequence becoming U+FFFD. Its end deletes it
## (a call into C++, `[FFI-17f]`). Not `Copy`, `Send` or `Sync`.
pub struct CppString:
    object: *mut u8
    data: unsafe extern "C" fn(*mut u8) -> *u8
    size: unsafe extern "C" fn(*mut u8) -> usize
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the string at `object`, with its thunks: what `import cpp` makes of a
    ## `std::string`.
    @safety("`object` is a live `std::string` nothing else owns, and the functions are its thunks")
    pub unsafe fn from_thunks(object: *mut u8, data: unsafe extern "C" fn(*mut u8) -> *u8, size: unsafe extern "C" fn(*mut u8) -> usize, destroy: unsafe extern "C" fn(*mut u8)) -> CppString:  # SAFETY: the caller keeps the obligation
        return CppString(object, data, size, destroy)

    ## The `std::string` itself, for a call into C++.
    pub fn as_ptr(self) -> *mut u8:
        return self.object

    pub fn len(self) -> usize:
        size = self.size
        unsafe:  # SAFETY(ffi): the string's own thunk
            return size(self.object)

    pub fn bytes(self) -> Span[u8]:
        data = self.data
        unsafe:  # SAFETY(ffi): the string's bytes, which the view borrows the string for
            return cpp_span(ref self, data(self.object), self.len())

    pub fn to_str(self) -> Result[str, Utf8Error]:
        return self.bytes().to_str()

    fn drop(mut self):
        destroy = self.destroy
        unsafe:  # SAFETY(ffi): the string's end, once
            destroy(self.object)

extend String implements From[CppString]:
    fn from(owned value: CppString) -> String:
        return cpp_string_copy(value)

## `[FFI-17a]` — `String.from(c)`: the bytes of a `std::string`, copied, each invalid sequence
## becoming U+FFFD. Compiler-known: `String.from` of a `CppString` is this.
fn cpp_string_copy(owned value: CppString) -> String:
    bytes = value.bytes()
    out = String()
    at = 0
    while at < bytes.len():
        lead = bytes[at] as u32
        width = 0
        if lead < 0x80:
            width = 1
        elif lead >= 0xC2 and lead < 0xE0:
            width = 2
        elif lead >= 0xE0 and lead < 0xF0:
            width = 3
        elif lead >= 0xF0 and lead < 0xF5:
            width = 4
        if width > 0 and at + width <= bytes.len():
            match bytes[at..at + width].to_str():
                Ok(text):
                    out.push_str(text)
                    at += width
                    continue
                Err(_):
                    pass
        out.push('\u{FFFD}')
        at += 1
    return out

## `[FFI-17a]`, `[FFI-17e]` — a C++ `std::shared_ptr<T>` Ember holds, never a `Shared` (`[WK-14]`):
## `get()` is the object it shares, borrowing the pointer; `clone()` and its end copy and destroy
## the `std::shared_ptr`, C++'s count changing in C++ (calls into C++, `[FFI-17f]`). Not `Copy`,
## `Send` or `Sync`.
pub struct CppShared[T]:
    object: *mut u8
    pointee: unsafe extern "C" fn(*mut u8) -> *mut T
    copy: unsafe extern "C" fn(*mut u8) -> *mut u8
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the `std::shared_ptr` at `object`, with its thunks: what `import cpp` makes
    ## of one.
    @safety("`object` is a live `std::shared_ptr<T>` nothing else owns, and the functions are its thunks")
    pub unsafe fn from_thunks(object: *mut u8, pointee: unsafe extern "C" fn(*mut u8) -> *mut T, copy: unsafe extern "C" fn(*mut u8) -> *mut u8, destroy: unsafe extern "C" fn(*mut u8)) -> CppShared[T]:  # SAFETY: the caller keeps the obligation
        return CppShared(object, pointee, copy, destroy)

    ## The `std::shared_ptr` itself, for a call into C++.
    pub fn as_ptr(self) -> *mut u8:
        return self.object

    ## The object it shares; an empty one panics.
    pub fn get(self) -> ref T:
        pointee = self.pointee
        address = null[*mut T]()
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            address = pointee(self.object)
        if address.is_null():
            panic("`CppShared.get`: the `std::shared_ptr` is empty")
        # The object the pointer shares, which the reference borrows the pointer for.
        return cpp_pointee(ref self, address)

    fn drop(mut self):
        destroy = self.destroy
        unsafe:  # SAFETY(ffi): this `std::shared_ptr`'s end, once
            destroy(self.object)

extend[T] CppShared[T] implements Clone:
    fn clone(self) -> CppShared[T]:
        copy = self.copy
        unsafe:  # SAFETY(ffi): a copy of the `std::shared_ptr`, which C++ counts
            return CppShared(copy(self.object), self.pointee, self.copy, self.destroy)

## `[FFI-17a]`, `[FFI-17e]` — a C++ `std::weak_ptr<T>` Ember holds, never a `Weak` (`[WK-14]`):
## `lock()` is a `CppShared[T]` of the object while C++'s count keeps it, `None` once it has gone,
## and `expired()` says which without making one. It carries the thunks of `std::shared_ptr<T>` for
## what `lock()` gives. Its copy (`clone()`) and its end are calls into C++ (`[FFI-17f]`). Not `Copy`,
## `Send` or `Sync`.
pub struct CppWeak[T]:
    object: *mut u8
    locker: unsafe extern "C" fn(*mut u8) -> *mut u8
    expiry: unsafe extern "C" fn(*mut u8) -> bool
    copy: unsafe extern "C" fn(*mut u8) -> *mut u8
    destroy: unsafe extern "C" fn(*mut u8)
    pointee: unsafe extern "C" fn(*mut u8) -> *mut T
    shared_copy: unsafe extern "C" fn(*mut u8) -> *mut u8
    shared_destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the `std::weak_ptr` at `object`, with its thunks and those of the
    ## `std::shared_ptr` its `lock()` gives: what `import cpp` makes of one.
    @safety("`object` is a live `std::weak_ptr<T>` nothing else owns, and the functions are its thunks and those of `std::shared_ptr<T>`")
    pub unsafe fn from_thunks(object: *mut u8, locker: unsafe extern "C" fn(*mut u8) -> *mut u8, expiry: unsafe extern "C" fn(*mut u8) -> bool, copy: unsafe extern "C" fn(*mut u8) -> *mut u8, destroy: unsafe extern "C" fn(*mut u8), pointee: unsafe extern "C" fn(*mut u8) -> *mut T, shared_copy: unsafe extern "C" fn(*mut u8) -> *mut u8, shared_destroy: unsafe extern "C" fn(*mut u8)) -> CppWeak[T]:  # SAFETY: the caller keeps the obligation
        return CppWeak(object, locker, expiry, copy, destroy, pointee, shared_copy, shared_destroy)

    ## The `std::weak_ptr` itself, for a call into C++.
    pub fn as_ptr(self) -> *mut u8:
        return self.object

    ## The object, shared again while C++'s count keeps it; `None` once it has gone.
    pub fn lock(self) -> Option[CppShared[T]]:
        locker = self.locker
        shared = null[*mut u8]()
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            shared = locker(self.object)
        if shared.is_null():
            return None
        unsafe:  # SAFETY: a `std::shared_ptr<T>` the thunk made for this value alone, with its thunks
            return Some(CppShared.from_thunks(shared, self.pointee, self.shared_copy, self.shared_destroy))

    ## Whether the object has gone.
    pub fn expired(self) -> bool:
        expiry = self.expiry
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            return expiry(self.object)

    fn drop(mut self):
        destroy = self.destroy
        unsafe:  # SAFETY(ffi): this `std::weak_ptr`'s end, once
            destroy(self.object)

extend[T] CppWeak[T] implements Clone:
    fn clone(self) -> CppWeak[T]:
        copy = self.copy
        unsafe:  # SAFETY(ffi): a copy of the `std::weak_ptr`, which C++ counts
            return CppWeak(copy(self.object), self.locker, self.expiry, self.copy, self.destroy, self.pointee, self.shared_copy, self.shared_destroy)

## `[FFI-17e]` — compiler-known: a C++ container's elements as a view, and a smart pointer's
## object as a reference, each borrowing `owner`.
fn cpp_span[O, T](owner: ref O, pointer: *mut T, count: usize) -> Span[T]:
    pass

fn cpp_span_mut[O, T](owner: ref mut O, pointer: *mut T, count: usize) -> MutSpan[T]:
    pass

fn cpp_pointee[O, T](owner: ref O, pointer: *mut T) -> ref T:
    pass

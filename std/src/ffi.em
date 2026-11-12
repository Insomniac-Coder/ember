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

## `[ERR-4]` — `unwrap` and `expect` on a `Result` panic with the error's `Display`.
extend NulError implements Display:
    ## It allocates as the `String` it writes to grows.
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(f"the text has a NUL byte at index {this.index}")

pub struct CString:
    bytes: String

extend StrView:
    ## It allocates a new `String`.
    pub fn to_cstring() -> Result[CString, NulError]:
        match this.find("\0"):
            Some(index):
                return Err(NulError(index as usize))
            None:
                bytes = this.to_string()
                bytes.push('\0')
                return Ok(CString(bytes))

extend String:
    ## It allocates a new `String`.
    pub fn to_cstring() -> Result[CString, NulError]:
        return this.as_str().to_cstring()

## `[TXT-7]` — text is UTF-8 everywhere; `WideString` is UTF-16 for the APIs that need it
## (Windows' `W` functions), converted explicitly: `s.to_wide()` encodes a `str` (an interior NUL
## is a `NulError` at its byte, as for `to_cstring`) and adds one 0 unit; `w.as_ptr()` is the
## terminated units for the call; `WideString.from_units(units)` takes C's units up to the first 0
## and `w.to_string()` decodes them, an unpaired surrogate becoming U+FFFD.
pub struct WideString:
    units: Array[u16]

    ## The units, without the terminating 0.
    pub fn units() -> Span[u16]:
        return this.units[0..this.units.len() - 1]

    ## The first unit's address: the units and their terminating 0, for a foreign call.
    pub fn as_ptr() -> *u16:
        return this.units.as_span().as_ptr()

    ## It allocates a new `Array`.
    pub static fn from_units(units: Span[u16]) -> WideString:
        kept: Array[u16] = []
        for unit in units:
            if unit == 0:
                break
            kept.push(owns unit)
        kept.push(owns 0)
        return WideString(kept)

    ## It allocates a new `String`.
    pub fn to_string() -> String:
        out = String()
        units = this.units()
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

extend StrView:
    ## It allocates a new `Array`.
    pub fn to_wide() -> Result[WideString, NulError]:
        units: Array[u16] = []
        for (at, c) in this.char_indices():
            code = c as u32
            if code == 0:
                return Err(NulError(at as usize))
            if code < 0x10000:
                units.push(owns (code as u16))
            else:
                rest = code - 0x10000
                units.push(owns ((0xD800 + (rest >> 10)) as u16))
                units.push(owns ((0xDC00 + (rest & 0x3FF)) as u16))
        units.push(owns 0)
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
## `[FFI-8]` — C's `long double`: opaque bytes of the target's size and alignment (8 on Windows,
## 16 on x86-64 and AArch64 Linux), passed and returned as C passes the type, with no arithmetic.
## The compiler gives it its own type; `u8` here only stands in.
pub type c_longdouble = u8
## `[FFI-8]` — C's `va_list`, as a parameter of it holds it: a value C made, received by a function
## C calls and passed on to one that reads it (`vsnprintf`), once; never constructed (`E5018`), a
## view of the call that passed it. The compiler gives it its own type; `u8` here only stands in.
pub type VaList = u8

## `[HR-39]` — the failure a foreign call returns as a value: a C++ exception its thunk caught
## (`CppError`), or a C function's status other than its success (`[FFI-16]`'s error types, which
## the compiler makes implement this). `?` turns one into `std.hot.ReloadError.Foreign` inside a
## `migrate_from`.
pub interface ForeignFailure: needs Error:
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
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        f.write_str(this.type_name)?
        f.write_str(": ")?
        return f.write_str(this.what)

extend CppError implements Error:
    pass

extend CppError implements ForeignFailure:
    pass

## `[FFI-36a]`, `[FFI-33b]` — what an `owns T` of a foreign type `T` is: the object's address, one
## word. The compiler makes one wherever a program writes `owns T` of a type an `extern` block
## declares (`adopt[T](h)`, a result contracted `returns_owned(destructor=f)`, a `std::unique_ptr`
## or a C++ class returned by value), shows it as `owns T`, and reads it as its `T`, as every `owns
## T`. A foreign type has one destructor (`@ffi(destroys(f))` on its declaration, or what each
## `returns_owned(destructor=f)` names), and the end calls it once, directly (`foreign_end`).
## `[FFI-36b]` — the runtime records every address an `owns T` of a foreign type holds, with the
## line that made it, so taking a live one again panics naming both lines, in every profile; the
## end, or a parameter contracted `owns`, takes the address out of the record. Not part of the
## language: a program writes `owns T`.
[T]
struct ForeignOwner:
    pointer: *mut T

    fn drop():
        foreign_end(this.pointer, 0)

## A foreign object may be ended on any thread unless its type's contract says `threads(creator)`
## (`ThreadForeignOwner`).
unsafe extend[T] ForeignOwner[T] implements Send:  # SAFETY: the type's contract binds it to no thread ([FFI-33b])
    pass

## `[FFI-33b]` — an `owns T` of a type whose contract says `threads(creator)`: two words, the
## object's address and the number of the thread that made it. Its end compares that number with
## the running thread's, in every profile, and stops the program on another (naming the type, the
## line that made it and both threads); safe code never hands one to another thread (it is not
## `Send`), so the stop backs unsafe code and C.
[T]
struct ThreadForeignOwner:
    pointer: *mut T
    creator: u32

    fn drop():
        foreign_end(this.pointer, this.creator)

## `[FFI-36a]`, `[FFI-36b]`, `[FFI-33b]` — compiler-known: the end of an `owns T` of a foreign
## type. The address leaves the runtime's record (checked against `creator`, the thread that made
## it, when that is not `0`), then `T`'s destructor is called on it.
[T]
fn foreign_end(pointer: *mut T, creator: u32):
    pass

## `[FFI-36a]` — take ownership of the foreign object at `h`: the `owns T` ends it with the one
## destructor of `T` (`@ffi(destroys(f))` on its declaration). Compiler-known: the destructor is
## read from that declaration, and one that says `@ffi(adopt=false)` is `E5052`.
@safety("`h` is a live foreign object of type `T` that nothing else owns, and `T`'s destructor frees it.")
[T]
pub unsafe fn adopt(h: *mut T) -> owns T:
    pass

## `[FFI-23]` — `Retained[ForeignCallback]` is the token a registering call returns for a callback C
## keeps (`[FFI-21]`): it keeps the callback's block on Ember's side while C holds its user data,
## and its end unregisters the callback through `destroys(f)` and then ends the block (the
## `ForeignCallback`'s end). Only the registering call makes one, and `Retained` holds nothing
## else: it gives no object away and keeps none for C. An Ember value C keeps is handed to a
## parameter whose contract says `owns` (the name ends and Ember frees nothing, `[FFI-11]`), or, for
## a C function whose declaration cannot say so, given with `into_raw()` inside `unsafe:`
## (`[FFI-54]`). `tok.ptr()` is the user data C was given. Compiler-known.
[T]
pub struct Retained:
    value: T

## `[FFI-21]` — what C keeps of a callback contracted `callback(retained, user_data=p)`: the
## callable, moved into an allocation of its own whose address C was given as the user data. Only
## the registering call makes one, held by the `Retained[ForeignCallback]` token it returns;
## `tok.ptr()` is that address. Its end unregisters the callback through the registering
## function's `destroys(f)`, then ends the callable and frees the allocation, once.
pub struct ForeignCallback:
    data: *mut u8
    release: extern "C" fn(*mut u8)

    ## It allocates when the callable's own `drop` does: ending it is a call through a C function
    ## pointer, whose effects nothing declares (`[STD-1]`).
    fn drop():
        release = this.release
        release(this.data)

## `[FFI-23]` (ODR-132) — the address of a class object, which `into_raw()` gives (`[FFI-54]`),
## for foreign code to keep or to be handed back: no count of its own (to C, one pointer). Two are
## equal when they are the address of one object.
@derive(Copy)
[C]
pub struct ObjectPtr:
    address: *mut u8

extend[C] ObjectPtr[C] implements Eq:
    ## Equal when they are the address of one object.
    fn eq(other: ObjectPtr[C]) -> bool:
        return this.address == other.address



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
[T]
pub struct CppVector:
    object: *mut u8
    data: unsafe extern "C" fn(*mut u8) -> *mut T
    size: unsafe extern "C" fn(*mut u8) -> usize
    push: unsafe extern "C" fn(*mut u8, *T)
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the vector at `object`, with its element type's thunks: what `import cpp`
    ## makes of a `std::vector<T>`.
    @safety("`object` is a live `std::vector<T>` nothing else owns, and the functions are the thunks of its element type")
    pub unsafe static fn from_thunks(object: *mut u8, data: unsafe extern "C" fn(*mut u8) -> *mut T, size: unsafe extern "C" fn(*mut u8) -> usize, push: unsafe extern "C" fn(*mut u8, *T), destroy: unsafe extern "C" fn(*mut u8)) -> CppVector[T]:
        return CppVector(object, data, size, push, destroy)

    ## The `std::vector` itself, for a call into C++.
    pub fn as_ptr() -> *mut u8:
        return this.object

    pub fn len() -> usize:
        size = this.size
        unsafe:  # SAFETY(ffi): the vector's own thunk, on the vector it came with
            return size(this.object)

    pub fn span() -> Span[T]:
        data = this.data
        unsafe:  # SAFETY(ffi): the vector's elements, which the view borrows the vector for
            return cpp_span(ref this, data(this.object), this.len())

    pub fn span_mut() -> MutSpan[T]:
        data = this.data
        count = this.len()
        unsafe:  # SAFETY(ffi): the vector's elements, viewed exclusively while the vector is
            return cpp_span_mut(ref mut this, data(this.object), count)

    pub fn push_back(value: T):
        push = this.push
        unsafe:  # SAFETY(ffi): the thunk copies the value into the vector
            push(this.object, ref_to_ptr(ref value))

    fn drop():
        destroy = this.destroy
        unsafe:  # SAFETY(ffi): the vector's end, once
            destroy(this.object)

extend[T: needs Copy] CppVector[T]:
    ## The elements, copied into an `Array`.
    pub fn to_array() -> Array[T]:
        out: Array[T] = []
        for item in this.span():
            out.push(owns item)
        return out

## `[FFI-17a]` — a C++ `std::function` Ember owns, which it does not call: what a result of one is
## with no overlay entry. Its end deletes it (a call into C++). With the entry's
## `@ffi(std_function)` the result is an `owned fn(A…) -> R` instead, whose capture this is
## (ODR-199). Not `Copy`, `Send` or `Sync`.
pub struct CppFunction:
    object: *mut u8
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the `std::function` at `object`, with its thunk: what `import cpp` makes of one.
    @safety("`object` is a live `std::function` nothing else owns, and `destroy` deletes it")
    pub unsafe static fn from_thunks(object: *mut u8, destroy: unsafe extern "C" fn(*mut u8)) -> CppFunction:
        return CppFunction(object, destroy)

    ## The `std::function` itself, for a call into C++.
    pub fn as_ptr() -> *mut u8:
        return this.object

    ## It allocates when the `std::function`'s destructor does: a call into C++ through a function
    ## pointer, whose effects nothing declares (`[STD-1]`).
    fn drop():
        destroy = this.destroy
        unsafe:  # SAFETY(ffi): the function's end, once
            destroy(this.object)

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
    pub unsafe static fn from_thunks(object: *mut u8, data: unsafe extern "C" fn(*mut u8) -> *u8, size: unsafe extern "C" fn(*mut u8) -> usize, destroy: unsafe extern "C" fn(*mut u8)) -> CppString:
        return CppString(object, data, size, destroy)

    ## The `std::string` itself, for a call into C++.
    pub fn as_ptr() -> *mut u8:
        return this.object

    ## It calls code whose effects are not declared, which may be code that allocates.
    pub fn len() -> usize:
        size = this.size
        unsafe:  # SAFETY(ffi): the string's own thunk
            return size(this.object)

    ## It calls code whose effects are not declared, which may be code that allocates.
    pub fn bytes() -> Span[u8]:
        data = this.data
        unsafe:  # SAFETY(ffi): the string's bytes, which the view borrows the string for
            return cpp_span(ref this, data(this.object), this.len())

    ## It calls code whose effects are not declared, which may be code that allocates.
    pub fn to_str() -> Result[StrView, Utf8Error]:
        return this.bytes().to_str()

    ## It calls code whose effects are not declared, which may be code that allocates.
    fn drop():
        destroy = this.destroy
        unsafe:  # SAFETY(ffi): the string's end, once
            destroy(this.object)

extend String implements From[CppString]:
    ## It allocates a new `String`.
    static fn from(owns value: CppString) -> String:
        return cpp_string_copy(owns value)

## `[FFI-17a]` — `String.from(c)`: the bytes of a `std::string`, copied, each invalid sequence
## becoming U+FFFD. Compiler-known: `String.from` of a `CppString` is this.
## It allocates a new `String`.
fn cpp_string_copy(owns value: CppString) -> String:
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
[T]
pub struct CppShared:
    object: *mut u8
    pointee: unsafe extern "C" fn(*mut u8) -> *mut T
    copy: unsafe extern "C" fn(*mut u8) -> *mut u8
    destroy: unsafe extern "C" fn(*mut u8)

    ## `[FFI-17a]` — the `std::shared_ptr` at `object`, with its thunks: what `import cpp` makes
    ## of one.
    @safety("`object` is a live `std::shared_ptr<T>` nothing else owns, and the functions are its thunks")
    pub unsafe static fn from_thunks(object: *mut u8, pointee: unsafe extern "C" fn(*mut u8) -> *mut T, copy: unsafe extern "C" fn(*mut u8) -> *mut u8, destroy: unsafe extern "C" fn(*mut u8)) -> CppShared[T]:
        return CppShared(object, pointee, copy, destroy)

    ## The `std::shared_ptr` itself, for a call into C++.
    pub fn as_ptr() -> *mut u8:
        return this.object

    ## The object it shares; an empty one panics (`try_get` gives an `Option`).
    pub fn get() -> ref T:
        pointee = this.pointee
        address = null[*mut T]()
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            address = pointee(this.object)
        if address.is_null():
            panic("`CppShared.get`: the `std::shared_ptr` is empty; use .try_get() for an Option")
        # The object the pointer shares, which the reference borrows the pointer for.
        return cpp_pointee(ref this, address)

    ## The object it shares, or `None` for an empty one (`get`'s twin, `[ERR-13]`).
    pub fn try_get() -> Option[ref T]:
        pointee = this.pointee
        address = null[*mut T]()
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            address = pointee(this.object)
        if address.is_null():
            return None
        return Some(cpp_pointee(ref this, address))

    fn drop():
        destroy = this.destroy
        unsafe:  # SAFETY(ffi): this `std::shared_ptr`'s end, once
            destroy(this.object)

extend[T] CppShared[T] implements Clone:
    fn clone() -> CppShared[T]:
        copy = this.copy
        unsafe:  # SAFETY(ffi): a copy of the `std::shared_ptr`, which C++ counts
            return CppShared(copy(this.object), this.pointee, this.copy, this.destroy)

## `[FFI-17a]`, `[FFI-17e]` — a C++ `std::weak_ptr<T>` Ember holds, never a `Weak` (`[WK-14]`):
## `lock()` is a `CppShared[T]` of the object while C++'s count keeps it, `None` once it has gone,
## and `expired()` says which without making one. It carries the thunks of `std::shared_ptr<T>` for
## what `lock()` gives. Its copy (`clone()`) and its end are calls into C++ (`[FFI-17f]`). Not `Copy`,
## `Send` or `Sync`.
[T]
pub struct CppWeak:
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
    pub unsafe static fn from_thunks(object: *mut u8, locker: unsafe extern "C" fn(*mut u8) -> *mut u8, expiry: unsafe extern "C" fn(*mut u8) -> bool, copy: unsafe extern "C" fn(*mut u8) -> *mut u8, destroy: unsafe extern "C" fn(*mut u8), pointee: unsafe extern "C" fn(*mut u8) -> *mut T, shared_copy: unsafe extern "C" fn(*mut u8) -> *mut u8, shared_destroy: unsafe extern "C" fn(*mut u8)) -> CppWeak[T]:
        return CppWeak(object, locker, expiry, copy, destroy, pointee, shared_copy, shared_destroy)

    ## The `std::weak_ptr` itself, for a call into C++.
    pub fn as_ptr() -> *mut u8:
        return this.object

    ## The object, shared again while C++'s count keeps it; `None` once it has gone.
    pub fn lock() -> Option[CppShared[T]]:
        locker = this.locker
        shared = null[*mut u8]()
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            shared = locker(this.object)
        if shared.is_null():
            return None
        unsafe:  # SAFETY: a `std::shared_ptr<T>` the thunk made for this value alone, with its thunks
            return Some(CppShared.from_thunks(shared, this.pointee, this.shared_copy, this.shared_destroy))

    ## Whether the object has gone.
    pub fn expired() -> bool:
        expiry = this.expiry
        unsafe:  # SAFETY(ffi): the pointer's own thunk
            return expiry(this.object)

    fn drop():
        destroy = this.destroy
        unsafe:  # SAFETY(ffi): this `std::weak_ptr`'s end, once
            destroy(this.object)

extend[T] CppWeak[T] implements Clone:
    fn clone() -> CppWeak[T]:
        copy = this.copy
        unsafe:  # SAFETY(ffi): a copy of the `std::weak_ptr`, which C++ counts
            return CppWeak(copy(this.object), this.locker, this.expiry, this.copy, this.destroy, this.pointee, this.shared_copy, this.shared_destroy)

## `[FFI-17e]` — compiler-known: a C++ container's elements as a view, and a smart pointer's
## object as a reference, each borrowing `owner`.
[O, T]
fn cpp_span(owner: ref O, pointer: *mut T, count: usize) -> Span[T]:
    pass

[O, T]
fn cpp_span_mut(ref mut owner: O, pointer: *mut T, count: usize) -> MutSpan[T]:
    pass

[O, T]
fn cpp_pointee(owner: ref O, pointer: *mut T) -> ref T:
    pass

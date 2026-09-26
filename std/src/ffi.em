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

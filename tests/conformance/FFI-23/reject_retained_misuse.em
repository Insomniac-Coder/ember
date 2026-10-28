#$ test: compile-fail
#$ rules: FFI-23, FFI-22, FFI-11
# `[FFI-23]` — `Retained` pins a class handle or a `Box`, and a token crosses
# to C only through a `retained` contract. `[FFI-22]` — what foreign code
# keeps is `Send`, unless the contract says which thread uses it. `tok.ptr()`
# is typed by what the token holds (ODR-132): one class's address is not
# another's, and a box's value is not a raw byte.

from std.ffi import Retained, ObjectPtr

class Noisy:
    name: str

class Quiet:
    name: str

fn address_of_another(token: Retained[Noisy]) -> ObjectPtr[Quiet]:
    return token.ptr()    #$ error[E2020]: expected `std.ffi.ObjectPtr[Quiet]`, found `std.ffi.ObjectPtr[Noisy]`

fn bytes_of_a_box(token: Retained[Box[i64]]) -> *mut u8:
    return token.ptr()    #$ error[E2020]: expected `*mut u8`, found `*mut i64`

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(object, retained, one))
    safe fn keep(object: Retained[Noisy])    #$ error[E7004]: `object` hands foreign code a `Noisy` to keep, and a `Noisy` is not `Send`
    @ffi(param(object, retained, one))
    safe fn keep_plain(object: i32)    #$ error[E5002]: `retained` contract for `object` needs `object: Retained[T]`
    safe fn keep_quietly(object: Retained[Box[i32]])    #$ error[E5002]: a `Retained` parameter crosses only as an address C keeps

fn main():
    number = Retained.pin(5)    #$ error[E2020]: `Retained` pins a class handle or a `Box`, not a `i64`

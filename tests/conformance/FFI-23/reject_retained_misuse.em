#$ test: compile-fail
#$ rules: FFI-23, FFI-22, FFI-11
# `[FFI-23]` — `Retained` pins a class handle or a `Box`, and a token crosses
# to C only through a `retained` contract. `[FFI-22]` — what foreign code
# keeps is `Send`, unless the contract says which thread uses it.

from std.ffi import Retained

class Noisy:
    name: str

unsafe extern "C":
    @ffi(param(object, retained, one))
    safe fn keep(object: Retained[Noisy])    #$ error[E7004]: `object` hands foreign code a `Noisy` to keep, and a `Noisy` is not `Send`
    @ffi(param(object, retained, one))
    safe fn keep_plain(object: i32)    #$ error[E5002]: `retained` contract for `object` needs `object: Retained[T]`
    safe fn keep_quietly(object: Retained[Box[i32]])    #$ error[E5002]: a `Retained` parameter crosses only as an address C keeps

fn main():
    number = Retained.pin(5)    #$ error[E2020]: `Retained` pins a class handle or a `Box`, not a `i64`

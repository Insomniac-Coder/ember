#$ test: compile-fail
#$ rules: FFI-11, FFI-36a
# ODR-269 — `handle` is a word of a type's contract (`@ffi(handle) type T`). A result of a handle
# type says who owns it and how long it lives with the words that do for any pointer
# (`returns_owned(destructor=f)`, `borrowed one from(p)`): a handle with no lifetime would let safe
# code pass C one C has freed, so `result(handle)` is `E5002`, and the result has no contract.

unsafe extern "C":
    @ffi(handle)
    type Device

    @ffi(result(handle))  #$ error[E5002]: `handle` is a word of a type's contract, not a result's
    safe fn device_open(id: i32) -> *mut Device  #$ error[E5002]: needs an `@ffi` contract for its pointer result

fn main():
    pass

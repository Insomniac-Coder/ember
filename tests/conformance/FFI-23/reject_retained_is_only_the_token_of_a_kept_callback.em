#$ test: compile-fail
#$ rules: FFI-23, FFI-11
# `[FFI-23]` — `Retained[ForeignCallback]` is only the token a registering call returns for a
# callback C keeps: `Retained` gives no object away and keeps none for C, so a token of anything
# else, and 0.9.10's `Retained.pin`, are not Ember. `[FFI-11]` — `retained` on a pointer that is
# no callback makes no safe parameter: C keeps Ember memory only when it is handed over (`owns`)
# or given with `into_raw()`, so the function stays an `unsafe fn` taking a raw pointer.

from std.ffi import Retained

class Noisy:
    name: StrView

fn kept(token: Retained[Noisy]):    #$ error[E2020]: `Retained` is only the token of a callback C keeps, `Retained[ForeignCallback]`, not a `Retained[Noisy]`
    pass

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(object, retained, one))
    safe fn keep(object: *mut u8)    #$ error[E5002]: `retained` on `object` makes no safe parameter: C keeps Ember memory only when it is handed over (`param(object, owns, one)`) or given with `into_raw()`; declare `keep` an `unsafe fn`

fn main():
    token = Retained.pin(owns Noisy(name = "kept"))    #$ error[E1010]: `Retained.pin` is gone: C keeps an Ember value handed to a parameter contracted `owns`, or given with `into_raw()` inside `unsafe:`

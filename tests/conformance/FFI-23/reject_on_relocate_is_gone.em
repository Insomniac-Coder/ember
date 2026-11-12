#$ test: compile-fail
#$ rules: FFI-23, HR-20
# `[HR-20]` — an object C holds (handed to a parameter contracted `owns`, or given with
# `into_raw()`) never moves, and a reload that would move it is refused naming it; 0.9.10's
# `Retained.pin(…, on_relocate=…)`, which let a reload move one and told C where, is gone with
# `Retained.pin` (`[FFI-23]`: `Retained` keeps no object for C).

from std.ffi import Retained

class Noisy:
    name: StrView

fn main():
    token = Retained.pin(owns Noisy(name = "a"), on_relocate=None)    #$ error[E1010]: `Retained.pin` is gone: C keeps an Ember value handed to a parameter contracted `owns`, or given with `into_raw()` inside `unsafe:`

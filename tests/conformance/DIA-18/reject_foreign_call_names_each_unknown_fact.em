#$ test: compile-fail
#$ rules: DIA-18, FFI-2, FFI-11c
#$ profiles: debug
#$ help: `fn abs_value(x)` in an `unsafe overlay c "facts.h":`
#$ help: `fn find_byte(data: TODO(ownership) TODO(count) TODO(nullable), key) -> TODO(ownership) TODO(count) TODO(nullable) TODO(lifetime)` in an `unsafe overlay c "facts.h":`
#$ help: `fn measure(text: TODO(ownership) TODO(count) TODO(nullable))` in an `unsafe overlay c "facts.h":`
# `[DIA-18]` — a call of a header's function that no overlay says anything of is refused for the
# facts its contract lacks: the error names each one and prints the overlay line that states them,
# every unknown fact a `TODO(…)` word to replace (`[FFI-11c]`). (A declaration written by hand says
# `safe fn` or `unsafe fn` itself, verdict 136.)

from std.ffi import c_char
import c "facts.h"

fn main():
    println(c.abs_value(-42))    #$ error[E3100]: unknown: `abs_value` is not asserted safe to call (`safe fn`)
    found = c.find_byte(null[*u8](), 3)    #$ error[E3100]: unknown: `data`: its ownership, count and nullability
    println(c.measure(null[*c_char]()))    #$ error[E3100]: unknown: `text`: its ownership, count and nullability

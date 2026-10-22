#$ test: run-pass
#$ rules: FFI-8, ENM-3
#$ profiles: debug, development, release
#$ stdout: MODE_AUTO 33 BIAS_LOW
#$ stdout: 1 33 1
#$ stdout: Mode(3) 3
# `[FFI-8]` — a bit-field of a C enum is read and written as the enum by `get_<f>`/`set_<f>`
# (through the enum's integer type, sign-extended where it is signed); a C enum holds every value
# of its type, so a value no variant names arrives as itself. Its neighbours keep their values, and
# C reads what Ember wrote. (Whether a C enum's bit-field is signed is the target's: its integer
# type, which `int` is for MSVC; the fields here are wide enough for either.)

import c "enum_bits.h"

fn main():
    s = c.Setting()
    unsafe:  # SAFETY(ffi): the functions take and return values
        s = c.setting_make()
    println(s.get_mode(), s.get_count(), s.get_bias())
    s.set_mode(c.Mode.MODE_ON)
    s.set_bias(c.Bias.BIAS_HIGH)
    unsafe:  # SAFETY(ffi): the functions take values
        println(c.setting_mode(s), c.setting_count(s), c.setting_bias(s))
    s.set_mode(c.Mode.from_repr(3).unwrap_or(c.Mode.MODE_OFF))
    unsafe:  # SAFETY(ffi): a value
        println(f"{s.get_mode():?}", c.setting_mode(s))

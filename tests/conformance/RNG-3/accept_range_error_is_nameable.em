#$ test: run-pass
#$ rules: RNG-3, RNG-10
# `[RNG-3]` writes the signature `T.checked(v) -> Result[T, RangeError]`, and
# this is that signature written out — Part IV.2a's own worked example. The
# name has to resolve while *signatures* are collected, before any body
# mentions `checked`, which is why `RangeError` is a prelude type rather than
# a `std.core` declaration: `checked` is a language-defined construction under
# `[RNG-10]`, not a library function, so its error type cannot wait on an
# import.

type Roughness = f32 in 0.0 ..= 1.0

fn from_slider(x: f32) -> Result[Roughness, RangeError]:
    return Roughness.checked(x)

fn main():
    match from_slider(0.5):
        Ok(_v) => println(1)
        Err(_e) => println(0)
    match from_slider(1.5):
        Ok(_v) => println(1)
        Err(_e) => println(0)
#$ stdout: 1
#$ 0

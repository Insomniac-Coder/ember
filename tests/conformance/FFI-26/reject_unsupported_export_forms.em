#$ test: compile-fail
#$ rules: FFI-26, MNG-2
#$ profiles: debug
# `[FFI-26]`, `[MNG-2]` — the symbol is set exactly, so it must be one a linker can hold; and a
# C symbol names one function, which a generic is not until its parameters are known.

@export("two words") #$ error[E0104]: `two words` cannot be a linker symbol
fn odd_name() -> i32:
    return 1

@export("identity") #$ error[E2060]: `identity` is generic, so its type parameters cannot be inferred for one C symbol
fn identity[T](value: T) -> T:
    return value

fn main():
    pass
